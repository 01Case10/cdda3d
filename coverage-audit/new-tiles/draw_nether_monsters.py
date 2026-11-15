#!/usr/bin/env python3
"""
Draw the missing nether + nether-labyrinth monster sprites from their game
descriptions. Sources (fork-src/cdda/data/json/monsters/nether.json and
netherum/labyrinth_monsters.json, CDDA 6a3e59e0): 52 ids from the coverage
audit's missing_shipped monster list with no existing sprite folder.

Design: flat UltiCa style -- dark outlines, flat fills, one highlight pass.
Bodies are fully opaque; glows and light cones are painted on separate
layers and composited with Image.alpha_composite. Sheet placement follows
the standing volume rule (resolved through copy-from chains):
  < 100 L -> pngs_normal_32x32, 100-400 L -> pngs_large_64x64,
  > 400 L -> pngs_giant_96x96.
Renderers are parameterized in unit space and drawn natively at each
sheet's resolution. Output is UltiCa decomposed layout (png + tile-entry
json + credits per folder).
"""
from PIL import Image, ImageDraw, ImageFilter
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

    def glow(self, cx, cy, r, color, alpha=100):
        layer = Image.new("RGBA", (self.size, self.size), (0, 0, 0, 0))
        ld = ImageDraw.Draw(layer)
        s = self.size
        for frac, a in ((1.0, alpha // 3), (0.66, alpha // 2), (0.36, alpha)):
            rr = r * frac
            ld.ellipse([(cx - rr) * s, (cy - rr) * s, (cx + rr) * s, (cy + rr) * s],
                       fill=color + (a,))
        layer = layer.filter(ImageFilter.GaussianBlur(max(1, s // 20)))
        self.img = Image.alpha_composite(self.img, layer)
        self.d = ImageDraw.Draw(self.img)

    def speckle(self, n, box, colors, rmin=0.006, rmax=0.015):
        for _ in range(n):
            x = self.rng.uniform(box[0], box[2])
            y = self.rng.uniform(box[1], box[3])
            r = self.rng.uniform(rmin, rmax)
            self.ell(x, y, r, r, self.rng.choice(colors))

    def curve(self, pts, fill, wunit=0.022, segs=16):
        """Smooth polyline through control points (Catmull-Rom-ish via lerp)."""
        out = []
        p = [pts[0]] + list(pts) + [pts[-1]]
        for i in range(len(p) - 3):
            for t_i in range(segs // 2):
                t = t_i / (segs // 2)
                x = 0.5 * ((2 * p[i + 1][0]) + (-p[i][0] + p[i + 2][0]) * t +
                           (2 * p[i][0] - 5 * p[i + 1][0] + 4 * p[i + 2][0] - p[i + 3][0]) * t * t +
                           (-p[i][0] + 3 * p[i + 1][0] - 3 * p[i + 2][0] + p[i + 3][0]) * t * t * t)
                y = 0.5 * ((2 * p[i + 1][1]) + (-p[i][1] + p[i + 2][1]) * t +
                           (2 * p[i][1] - 5 * p[i + 1][1] + 4 * p[i + 2][1] - p[i + 3][1]) * t * t +
                           (-p[i][1] + 3 * p[i + 1][1] - 3 * p[i + 2][1] + p[i + 3][1]) * t * t * t)
                out.append((x, y))
        out.append(pts[-1])
        self.line(out, fill, wunit)


def humanoid(ctx, body, dark=DARK, hi=None, head=None, skin=None,
             arm_pose="down", distort=0.0, lean=0.0, leg_spread=0.09):
    """Standing humanoid in unit space; optional sine distortion of x by y."""
    def T(x, y):
        return (x + distort * math.sin(y * 8.0) + lean * (y - 0.3), y)
    head = head or body
    # legs
    for sx in (-1, 1):
        hip = T(0.5 + sx * 0.055, 0.56)
        knee = T(0.5 + sx * (0.055 + leg_spread * 0.4), 0.74)
        foot = T(0.5 + sx * (0.055 + leg_spread), 0.93)
        ctx.line([hip, knee, foot], dark, 0.085)
        ctx.line([hip, knee, foot], body, 0.062)
    # torso
    torso = [T(0.385, 0.30), T(0.615, 0.30), T(0.575, 0.58), T(0.425, 0.58)]
    ctx.poly(torso, dark)
    inner = [(0.5 + (x - 0.5) * 0.82, y + 0.012) for x, y in torso]
    ctx.poly(inner, body)
    if hi:
        ctx.poly([(x, y) for x, y in
                  [(0.42, 0.33), (0.50, 0.33), (0.485, 0.52), (0.435, 0.52)]], hi)
    # arms
    for sx in (-1, 1):
        sh = T(0.5 + sx * 0.125, 0.315)
        if arm_pose == "down":
            el = T(0.5 + sx * 0.185, 0.45); ha = T(0.5 + sx * 0.165, 0.60)
        elif arm_pose == "reach":
            el = T(0.5 + sx * 0.21, 0.38); ha = T(0.5 + sx * 0.27, 0.30)
        elif arm_pose == "long":  # one arm (right) hangs too long
            if sx == 1:
                el = T(0.5 + sx * 0.17, 0.48); ha = T(0.5 + sx * 0.15, 0.74)
            else:
                el = T(0.5 + sx * 0.185, 0.45); ha = T(0.5 + sx * 0.165, 0.60)
        ctx.line([sh, el, ha], dark, 0.07)
        ctx.line([sh, el, ha], body, 0.05)
        ctx.ell(ha[0], ha[1], 0.032, 0.032, skin or body, dark)
    # head
    hx, hy = T(0.5, 0.185)
    ctx.ell(hx, hy, 0.088, 0.098, dark)
    ctx.ell(hx, hy, 0.074, 0.084, head)


# ---------------------------------------------------------------- robots ---

def draw_fixbot(ctx, body, hi, eye_xy=(0.5, 0.16), pupil=(0.0, 0.0),
                arms="even", badge=None, haywire=False):
    # three bulbous limbs
    for lx in (0.30, 0.50, 0.70):
        ctx.ell(lx, 0.875, 0.085, 0.075, (90, 26, 26, 255), DARK)
        ctx.ell(lx, 0.875, 0.062, 0.052, body)
    # round body
    ctx.ell(0.5, 0.56, 0.30, 0.27, DARK)
    ctx.ell(0.5, 0.56, 0.275, 0.245, body)
    ctx.ell(0.415, 0.44, 0.13, 0.09, hi)  # highlight
    # arms above the body (articulated, emotive)
    if arms == "even":
        poses = [(-1, [(0.26, 0.42), (0.16, 0.28), (0.22, 0.16)]),
                 (1, [(0.74, 0.42), (0.84, 0.28), (0.78, 0.16)])]
    else:  # flailing asymmetric
        poses = [(-1, [(0.26, 0.42), (0.12, 0.34), (0.10, 0.18)]),
                 (1, [(0.74, 0.42), (0.88, 0.22), (0.80, 0.10)])]
    for sx, pts in poses:
        ctx.line(pts, DARK, 0.075)
        ctx.line(pts, body, 0.05)
        for jx, jy in pts[1:]:
            ctx.ell(jx, jy, 0.038, 0.038, hi, DARK)
        hx, hy = pts[-1]
        for f in (-0.03, 0.0, 0.03):  # little gripper fingers
            ctx.line([(hx, hy), (hx + f, hy - 0.045)], DARK, 0.02)
    # swiveling eye on a stalk
    ctx.line([(0.5, 0.33), eye_xy], DARK, 0.06)
    ctx.line([(0.5, 0.33), eye_xy], body, 0.04)
    ctx.ell(eye_xy[0], eye_xy[1], 0.105, 0.105, DARK)
    ctx.ell(eye_xy[0], eye_xy[1], 0.088, 0.088, (238, 238, 230, 255))
    ctx.ell(eye_xy[0] + pupil[0], eye_xy[1] + pupil[1], 0.042, 0.042, (30, 30, 60, 255))
    ctx.ell(eye_xy[0] + pupil[0] - 0.014, eye_xy[1] + pupil[1] - 0.014, 0.014, 0.014,
            (255, 255, 255, 255))
    if badge == "cross":
        ctx.rect(0.435, 0.52, 0.565, 0.65, (240, 238, 232, 255), DARK)
        ctx.rect(0.485, 0.535, 0.515, 0.635, (214, 40, 50, 255))
        ctx.rect(0.45, 0.57, 0.55, 0.60, (214, 40, 50, 255))
    if haywire:  # motion arcs + beep marks
        for r, a0, a1 in ((0.36, 200, 300), (0.42, 240, 330)):
            ctx.arc(0.5, 0.56, r, r * 0.9, a0, a1, (120, 120, 130, 255), 0.016)
        for bx, by in ((0.14, 0.10), (0.86, 0.06)):
            ctx.line([(bx, by), (bx + 0.03, by - 0.03)], (240, 200, 60, 255), 0.018)
            ctx.line([(bx, by - 0.03), (bx + 0.03, by)], (240, 200, 60, 255), 0.018)


def fixbot(ctx):
    draw_fixbot(ctx, (198, 42, 46, 255), (232, 96, 92, 255))


def fixbot_haywire(ctx):
    draw_fixbot(ctx, (198, 42, 46, 255), (232, 96, 92, 255),
                eye_xy=(0.56, 0.13), pupil=(0.03, -0.02), arms="wild", haywire=True)


def medbot(ctx):
    draw_fixbot(ctx, (232, 130, 160, 255), (246, 180, 198, 255), badge="cross")


def spotbot(ctx):
    draw_fixbot(ctx, (226, 96, 92, 255), (240, 150, 140, 255), eye_xy=(0.44, 0.15))
    # flashlight cone from the eye, composited overlay
    layer = Image.new("RGBA", (ctx.size, ctx.size), (0, 0, 0, 0))
    ld = ImageDraw.Draw(layer)
    ld.polygon(ctx.X([(0.44, 0.15), (0.06, 0.44), (0.20, 0.60)]), fill=(255, 244, 190, 70))
    layer = layer.filter(ImageFilter.GaussianBlur(max(1, ctx.size // 32)))
    ctx.img = Image.alpha_composite(ctx.img, layer)
    ctx.d = ImageDraw.Draw(ctx.img)
    ctx.ell(0.44, 0.15, 0.045, 0.045, (255, 250, 210, 255))


def robot_arm(ctx):
    s = ctx.size
    # conveyor belt hint along the bottom
    ctx.rect(0.02, 0.90, 0.98, 0.97, (52, 50, 58, 255), DARK)
    for i in range(6):
        x = 0.06 + i * 0.155
        ctx.line([(x, 0.965), (x + 0.06, 0.905)], (96, 94, 104, 255), 0.018)
    # floor mount + base
    ctx.poly([(0.36, 0.90), (0.64, 0.90), (0.58, 0.76), (0.42, 0.76)],
             (120, 116, 110, 255), DARK)
    ctx.ell(0.5, 0.90, 0.30, 0.035, (84, 80, 78, 255), DARK)
    orange, dark_o, hi = (232, 138, 34, 255), (168, 92, 22, 255), (250, 186, 92, 255)
    # lower arm up-left, upper arm up-right (articulated elbow)
    base = (0.5, 0.78); elbow = (0.335, 0.44); wrist = (0.60, 0.20)
    ctx.line([base, elbow], DARK, 0.20)
    ctx.line([base, elbow], dark_o, 0.155)
    ctx.line([elbow, wrist], DARK, 0.165)
    ctx.line([elbow, wrist], orange, 0.125)
    ctx.line([(0.47, 0.70), (0.37, 0.50)], hi, 0.045)  # highlight pass
    for jx, jy, r in ((0.5, 0.78, 0.105), (0.335, 0.44, 0.095), (0.60, 0.20, 0.075)):
        ctx.ell(jx, jy, r, r, DARK)
        ctx.ell(jx, jy, r * 0.72, r * 0.72, hi)
        ctx.ell(jx, jy, r * 0.30, r * 0.30, (70, 66, 66, 255))
    # illuminated eye on the upper arm
    ctx.glow(0.47, 0.315, 0.075, (120, 230, 255), 130)
    ctx.ell(0.47, 0.315, 0.030, 0.030, (190, 245, 255, 255), DARK)
    # over-articulated fingers from the wrist
    for fx, fy, bend in ((0.70, 0.10, 1), (0.78, 0.19, 1), (0.74, 0.29, -1), (0.64, 0.335, -1)):
        mid = (fx - 0.045 * bend, fy - 0.01)
        tip = (fx - 0.085 * bend, fy + 0.045)
        ctx.line([wrist, mid, tip], DARK, 0.052)
        ctx.line([wrist, mid, tip], orange, 0.032)
        ctx.ell(mid[0], mid[1], 0.026, 0.026, hi, DARK)
    # alien symbol traces it has been drawing on the floor
    for pts in ([(0.10, 0.82), (0.16, 0.76), (0.21, 0.83)],
                [(0.82, 0.80), (0.88, 0.86), (0.80, 0.87)]):
        ctx.line(pts, (240, 200, 90, 255), 0.014)


def bot_printer(ctx, body, hi, mouth_w=0.30, sparks=False):
    # pillar of technology on prebuilt tracks
    ctx.rect(0.30, 0.10, 0.70, 0.86, DARK)
    ctx.rect(0.325, 0.125, 0.675, 0.835, body)
    ctx.rect(0.35, 0.15, 0.65, 0.34, hi)  # upper panel highlight
    for tx in (0.375, 0.50, 0.625):  # tracks traced over the device
        ctx.line([(tx, 0.14), (tx, 0.82)], (40, 42, 54, 255), 0.016)
    # multitude of tiny crawling robots on the tracks
    for _ in range(26):
        x = ctx.rng.choice((0.375, 0.50, 0.625)) + ctx.rng.uniform(-0.012, 0.012)
        y = ctx.rng.uniform(0.16, 0.62)
        ctx.ell(x, y, 0.016, 0.013, (226, 228, 220, 255), DARK)
    # massive mechanical mouth, vomiting little machines
    mw = mouth_w
    ctx.poly([(0.5 - mw, 0.60), (0.5 + mw, 0.60), (0.5 + mw * 0.7, 0.80),
              (0.5 - mw * 0.7, 0.80)], (26, 24, 30, 255), DARK)
    for i in range(5):  # jaw teeth
        x = 0.5 - mw * 0.8 + i * mw * 0.4
        ctx.poly([(x, 0.615), (x + mw * 0.16, 0.615), (x + mw * 0.08, 0.665)],
                 (210, 208, 198, 255))
    for bx, by in ((0.42, 0.87), (0.53, 0.90), (0.60, 0.855)):  # vomited bots
        ctx.ell(bx, by, 0.035, 0.028, (200, 202, 196, 255), DARK)
        ctx.ell(bx - 0.012, by - 0.006, 0.007, 0.007, (240, 90, 80, 255))
    ctx.speckle(10, (0.34, 0.38, 0.66, 0.55),
                [(240, 220, 120, 255), (140, 220, 255, 255)], 0.005, 0.010)
    if sparks:
        for _ in range(8):
            x = ctx.rng.uniform(0.28, 0.72); y = ctx.rng.uniform(0.08, 0.4)
            ctx.line([(x, y), (x + ctx.rng.uniform(-0.05, 0.05), y + 0.05)],
                     (255, 190, 70, 255), 0.012)


def sparkgenerator(ctx):
    bot_printer(ctx, (66, 96, 196, 255), (110, 140, 224, 255))


def sparkgenerator_haywire(ctx):
    bot_printer(ctx, (198, 52, 48, 255), (228, 104, 92, 255), mouth_w=0.36, sparks=True)


def ribcage(ctx, steel, hi, eyes=False):
    # jagged steel ribs crossing back and forth
    for i, off in enumerate((-0.20, -0.10, 0.0, 0.10, 0.20)):
        flip = 1 if i % 2 == 0 else -1
        ctx.arc(0.5 + off * 0.4, 0.52, 0.30 - abs(off) * 0.35, 0.40,
                100 if flip > 0 else 260, 260 if flip > 0 else 80, DARK, 0.075)
        ctx.arc(0.5 + off * 0.4, 0.52, 0.30 - abs(off) * 0.35, 0.40,
                100 if flip > 0 else 260, 260 if flip > 0 else 80, steel, 0.05)
    ctx.line([(0.5, 0.08), (0.5, 0.96)], DARK, 0.05)  # spine column
    ctx.line([(0.5, 0.08), (0.5, 0.96)], hi, 0.028)
    # uncountable microscopic robots coating the surface
    ctx.speckle(90, (0.16, 0.14, 0.84, 0.90),
                [(228, 230, 224, 255), (150, 200, 235, 255), (90, 96, 110, 255)],
                0.004, 0.009)
    # beautiful churned symbols
    for sx, sy in ((0.30, 0.30), (0.68, 0.42), (0.38, 0.68), (0.62, 0.78)):
        ctx.line([(sx, sy), (sx + 0.05, sy + 0.04), (sx + 0.10, sy)],
                 (240, 214, 120, 255), 0.014)
    if eyes:  # great glowing eyes have formed
        for ex in (0.36, 0.64):
            ctx.glow(ex, 0.34, 0.11, (255, 120, 90), 120)
            ctx.ell(ex, 0.34, 0.045, 0.055, (255, 176, 130, 255), DARK)
            ctx.ell(ex, 0.34, 0.016, 0.026, (60, 16, 10, 255))


def sparkgenerator_aux(ctx):
    ribcage(ctx, (140, 168, 196, 255), (196, 216, 236, 255))


def sparkgenerator_aux_haywire(ctx):
    ribcage(ctx, (206, 120, 118, 255), (236, 170, 162, 255), eyes=True)


def surgicalbot(ctx, assist=False):
    bone, bone_hi, dark_b = (216, 208, 186, 255), (238, 232, 214, 255), (150, 142, 122, 255)
    # roller base
    ctx.rect(0.34, 0.86, 0.66, 0.95, dark_b, DARK)
    for bx in (0.38, 0.47, 0.56):
        ctx.ell(bx + 0.03, 0.905, 0.028, 0.028, (96, 90, 76, 255), DARK)
    # four arms first (behind torso)
    arm_sets = [(-1, 0.34, 0.52), (-1, 0.40, 0.66), (1, 0.34, 0.52), (1, 0.40, 0.66)]
    for sx, ay, hy in arm_sets:
        sh = (0.5 + sx * 0.13, ay)
        el = (0.5 + sx * 0.24, ay + 0.10)
        ha = (0.5 + sx * 0.20, hy)
        if assist and sx == 1 and ay == 0.40:
            ha = (0.5 + sx * 0.26, hy - 0.06)
        ctx.line([sh, el, ha], DARK, 0.062)
        ctx.line([sh, el, ha], bone, 0.042)
        for f in (-0.025, 0.0, 0.025):  # over-articulated fingers
            ctx.line([ha, (ha[0] + f, ha[1] + 0.045)], DARK, 0.013)
    if assist:  # tray with instruments
        ctx.rect(0.60, 0.52, 0.86, 0.555, (188, 190, 186, 255), DARK)
        ctx.line([(0.65, 0.52), (0.68, 0.485)], (120, 124, 128, 255), 0.014)
        ctx.line([(0.74, 0.52), (0.79, 0.49)], (120, 124, 128, 255), 0.014)
    # torso clad in bone-like material
    ctx.poly([(0.375, 0.30), (0.625, 0.30), (0.585, 0.88), (0.415, 0.88)], DARK)
    ctx.poly([(0.395, 0.315), (0.605, 0.315), (0.57, 0.865), (0.43, 0.865)], bone)
    ctx.poly([(0.42, 0.34), (0.50, 0.34), (0.49, 0.60), (0.435, 0.60)], bone_hi)
    for ry in (0.48, 0.62, 0.76):  # rib-like panel seams
        ctx.arc(0.5, ry, 0.085, 0.045, 15, 165, dark_b, 0.016)
    # featureless head: surgical mask + face shield
    ctx.ell(0.5, 0.185, 0.088, 0.098, DARK)
    ctx.ell(0.5, 0.185, 0.074, 0.084, bone)
    ctx.rect(0.435, 0.205, 0.565, 0.265, (168, 205, 214, 255), DARK)  # mask
    ctx.line([(0.435, 0.22), (0.565, 0.22)], (120, 160, 168, 255), 0.010)
    ctx.rect(0.425, 0.105, 0.575, 0.20, (206, 228, 232, 255), (120, 130, 134, 255))  # shield


def surgicalbot_1_1(ctx):
    surgicalbot(ctx, assist=False)


def surgicalbot_assist(ctx):
    surgicalbot(ctx, assist=True)


def vine_turret(ctx):
    green, green_d, flesh = (74, 138, 62, 255), (44, 92, 40, 255), (178, 110, 104, 255)
    # ceiling anchor
    ctx.rect(0.18, 0.0, 0.82, 0.07, (40, 44, 40, 255), DARK)
    # massive tangle of hanging vines
    for i in range(15):
        x0 = 0.22 + i * 0.04 + ctx.rng.uniform(-0.015, 0.015)
        sway = ctx.rng.uniform(-0.10, 0.10)
        y1 = ctx.rng.uniform(0.45, 0.78)
        ctx.curve([(x0, 0.04), (x0 + sway, y1 * 0.5), (x0 - sway * 0.5, y1)],
                  green_d if i % 3 else green, 0.030)
    for i in range(6):  # woody tendrils + fleshy sinew strands
        x0 = ctx.rng.uniform(0.3, 0.7)
        ctx.curve([(x0, 0.05), (x0 + 0.06, 0.3), (x0 - 0.04, 0.55)],
                  (128, 96, 58, 255) if i % 2 else flesh, 0.020)
    # dripping blood and acrid ichor
    for dx, dy, col in ((0.30, 0.62, (150, 30, 30, 255)), (0.44, 0.72, (168, 160, 60, 255)),
                        (0.63, 0.66, (150, 30, 30, 255)), (0.72, 0.58, (168, 160, 60, 255))):
        ctx.line([(dx, dy - 0.09), (dx, dy)], col, 0.012)
        ctx.ell(dx, dy + 0.012, 0.014, 0.02, col)
    # the dangling metal gun with copper tubes
    ctx.line([(0.5, 0.30), (0.5, 0.44)], (96, 96, 100, 255), 0.03)
    ctx.rect(0.415, 0.44, 0.585, 0.56, (104, 108, 114, 255), DARK)  # receiver
    ctx.rect(0.475, 0.56, 0.525, 0.78, (70, 72, 78, 255), DARK)     # barrel
    ctx.rect(0.462, 0.76, 0.538, 0.815, (52, 54, 60, 255), DARK)    # muzzle
    ctx.curve([(0.42, 0.47), (0.34, 0.55), (0.40, 0.66)], (186, 110, 54, 255), 0.022)
    ctx.curve([(0.58, 0.47), (0.66, 0.55), (0.60, 0.66)], (186, 110, 54, 255), 0.022)
    ctx.ell(0.5, 0.50, 0.028, 0.028, (240, 80, 60, 255), DARK)  # sensor eye
    ctx.speckle(14, (0.25, 0.1, 0.75, 0.5), [green, (120, 170, 90, 255)], 0.008, 0.02)


def plantborg(ctx, engorged=False):
    metal, metal_hi = (168, 172, 178, 255), (208, 212, 218, 255)
    vine = (96, 158, 84, 255) if not engorged else (78, 138, 66, 255)
    vine_d = (58, 108, 52, 255)
    lean = 0.05 if engorged else 0.0
    humanoid(ctx, metal, hi=metal_hi, head=metal_hi, lean=lean,
             arm_pose="reach" if engorged else "down")
    # plant-like plastic wiring tangles over the torso and limbs
    n = 16 if engorged else 9
    for _ in range(n):
        x0 = ctx.rng.uniform(0.38, 0.62); y0 = ctx.rng.uniform(0.30, 0.60)
        pts = [(x0, y0)]
        for k in range(3):
            pts.append((x0 + ctx.rng.uniform(-0.13, 0.13), y0 + 0.09 * (k + 1)))
        ctx.curve(pts, vine if _ % 2 else vine_d, 0.030 if engorged else 0.022)
    if engorged:  # vines pool at the feet, weighing it down
        for _ in range(6):
            x0 = ctx.rng.uniform(0.3, 0.7)
            ctx.curve([(x0, 0.86), (x0 + ctx.rng.uniform(-0.08, 0.08), 0.93),
                       (x0 + ctx.rng.uniform(-0.12, 0.12), 0.97)], vine_d, 0.026)
    # softly bioluminescent eyes
    for ex in (0.468, 0.532):
        ctx.glow(ex, 0.185, 0.045, (140, 255, 190), 110)
        ctx.ell(ex, 0.185, 0.013, 0.016, (190, 255, 215, 255))
    # panel seams
    ctx.line([(0.40, 0.44), (0.60, 0.44)], (110, 114, 120, 255), 0.012)


def plantborg_2_1(ctx):
    plantborg(ctx, engorged=False)


def plantborg_2_2(ctx):
    plantborg(ctx, engorged=True)


def vinewalker(ctx):
    rust, rust_d = (150, 104, 66, 255), (104, 70, 42, 255)
    tube, tube_d = (122, 156, 96, 255), (84, 116, 66, 255)
    # four lanky thorn-covered tendril legs
    for fx, spread in ((0.16, -1), (0.36, -1), (0.64, 1), (0.84, 1)):
        top = (0.5 + (fx - 0.5) * 0.35, 0.42)
        mid = (fx + spread * 0.03, 0.68)
        foot = (fx + spread * 0.05, 0.95)
        ctx.curve([top, mid, foot], tube_d, 0.045)
        ctx.curve([top, mid, foot], tube, 0.028)
        for t in (0.3, 0.5, 0.7, 0.85):  # thorns
            x = top[0] + (foot[0] - top[0]) * t
            y = top[1] + (foot[1] - top[1]) * t
            ctx.line([(x, y), (x + spread * 0.035, y - 0.02)], rust_d, 0.014)
    # loosely humanoid torso: jumble of rusting wire and plant tubes
    ctx.ell(0.5, 0.33, 0.155, 0.16, DARK)
    ctx.ell(0.5, 0.33, 0.135, 0.14, (128, 118, 96, 255))
    for _ in range(7):
        x0 = ctx.rng.uniform(0.40, 0.60); y0 = ctx.rng.uniform(0.22, 0.42)
        ctx.curve([(x0, y0), (x0 + ctx.rng.uniform(-0.08, 0.08), y0 + 0.08),
                   (x0 + ctx.rng.uniform(-0.06, 0.06), y0 + 0.15)],
                  rust if _ % 2 else tube_d, 0.018)
    ctx.ell(0.455, 0.28, 0.045, 0.05, tube)   # plant tube bulges
    ctx.ell(0.55, 0.37, 0.04, 0.045, rust)
    # face: clump of hanging wires and dead vines, breathing like foul lungs
    for i in range(9):
        x0 = 0.435 + i * 0.016
        ln = 0.10 + (0.035 if i % 2 else 0.0)
        ctx.curve([(x0, 0.155), (x0 + 0.008, 0.155 + ln * 0.5), (x0 - 0.006, 0.155 + ln)],
                  (88, 76, 60, 255) if i % 3 else tube_d, 0.013)
    ctx.ell(0.5, 0.145, 0.075, 0.045, (96, 86, 70, 255), DARK)  # head clump base


def wells_machine(ctx):
    brass, brass_d, brass_hi = (178, 134, 54, 255), (128, 92, 34, 255), (226, 182, 96, 255)
    # two legs hinted at the bottom
    for sx in (-1, 1):
        ctx.poly([(0.5 + sx * 0.10, 0.78), (0.5 + sx * 0.24, 0.78),
                  (0.5 + sx * 0.30, 0.97), (0.5 + sx * 0.16, 0.97)], brass_d, DARK)
    # brazen armored hull
    hull = [(0.16, 0.30), (0.84, 0.30), (0.90, 0.62), (0.76, 0.82),
            (0.24, 0.82), (0.10, 0.62)]
    ctx.poly(hull, DARK)
    inner = [(0.5 + (x - 0.5) * 0.90, 0.56 + (y - 0.56) * 0.90) for x, y in hull]
    ctx.poly(inner, brass)
    ctx.poly([(0.20, 0.34), (0.46, 0.34), (0.42, 0.55), (0.22, 0.55)], brass_hi)
    # rivets
    for rx, ry in ((0.20, 0.36), (0.80, 0.36), (0.16, 0.60), (0.84, 0.60),
                   (0.30, 0.78), (0.70, 0.78)):
        ctx.ell(rx, ry, 0.014, 0.014, brass_d, DARK)
    # massive aperture eye
    ctx.ell(0.5, 0.44, 0.175, 0.175, DARK)
    ctx.ell(0.5, 0.44, 0.155, 0.155, (214, 178, 96, 255))
    for i in range(6):  # aperture blades
        a = i * math.pi / 3 + 0.3
        x1 = 0.5 + 0.145 * math.cos(a); y1 = 0.44 + 0.145 * math.sin(a)
        x2 = 0.5 + 0.075 * math.cos(a + 0.7); y2 = 0.44 + 0.075 * math.sin(a + 0.7)
        ctx.line([(x1, y1), (x2, y2)], brass_d, 0.020)
    ctx.ell(0.5, 0.44, 0.072, 0.072, (26, 22, 30, 255))
    ctx.ell(0.5, 0.44, 0.030, 0.030, (200, 40, 36, 255))  # scanning core
    ctx.ell(0.472, 0.412, 0.016, 0.016, (255, 240, 220, 255))
    # arsenal bristling beneath the eye
    for bx in (0.30, 0.40, 0.60, 0.70):
        ctx.rect(bx - 0.016, 0.66, bx + 0.016, 0.80, (66, 62, 60, 255), DARK)
    for bx in (0.35, 0.50, 0.65):  # missile pods
        ctx.rect(bx - 0.030, 0.63, bx + 0.030, 0.72, brass_d, DARK)
        ctx.ell(bx, 0.665, 0.011, 0.011, (240, 120, 60, 255))
    # rust streaks against the brazen armor
    ctx.speckle(16, (0.2, 0.55, 0.8, 0.8), [(146, 84, 40, 255), (110, 66, 34, 255)],
                0.006, 0.014)


def wells_machine_leg(ctx):
    brass, brass_d, brass_hi = (178, 134, 54, 255), (128, 92, 34, 255), (226, 182, 96, 255)
    # one of the warlord's three legs, thin brass plates over a dark frame
    hip = (0.40, 0.06); knee = (0.60, 0.46); ankle = (0.46, 0.82)
    ctx.line([hip, knee, ankle], DARK, 0.30)
    ctx.line([hip, knee, ankle], (70, 62, 54, 255), 0.22)
    # thigh plates
    for i in range(3):
        t = i / 3.0
        x = hip[0] + (knee[0] - hip[0]) * t + 0.02
        y = hip[1] + (knee[1] - hip[1]) * t
        ctx.poly([(x - 0.10, y), (x + 0.10, y + 0.02), (x + 0.085, y + 0.115),
                  (x - 0.085, y + 0.10)], brass if i % 2 else brass_hi, DARK)
    # knee joint
    ctx.ell(knee[0], knee[1], 0.105, 0.105, DARK)
    ctx.ell(knee[0], knee[1], 0.078, 0.078, brass_d)
    ctx.ell(knee[0], knee[1], 0.032, 0.032, (60, 54, 48, 255))
    # shin plates
    for i in range(3):
        t = i / 3.0
        x = knee[0] + (ankle[0] - knee[0]) * t - 0.01
        y = knee[1] + (ankle[1] - knee[1]) * t + 0.05
        ctx.poly([(x - 0.085, y), (x + 0.085, y - 0.01), (x + 0.07, y + 0.10),
                  (x - 0.07, y + 0.105)], brass_hi if i % 2 else brass, DARK)
    # broad foot
    ctx.poly([(0.30, 0.86), (0.66, 0.86), (0.74, 0.96), (0.24, 0.96)], brass_d, DARK)
    ctx.poly([(0.34, 0.885), (0.62, 0.885), (0.68, 0.945), (0.29, 0.945)], brass)
    for tx in (0.30, 0.48, 0.66):  # toe claws
        ctx.poly([(tx - 0.035, 0.955), (tx + 0.035, 0.955), (tx, 0.995)], (90, 78, 44, 255), DARK)
    ctx.speckle(12, (0.3, 0.15, 0.7, 0.85), [(146, 84, 40, 255)], 0.005, 0.012)


# ------------------------------------------------------- nether humanoids ---

def pale_reflection(ctx):
    # barely real: a faint, wispy near-silhouette of a person
    gray, gray_d = (196, 196, 202, 255), (140, 140, 148, 255)
    humanoid(ctx, gray, dark=gray_d, hi=(216, 216, 222, 255), head=gray)
    for _ in range(7):  # wisps trailing off the feet and hands
        x0 = ctx.rng.uniform(0.36, 0.64)
        ctx.curve([(x0, 0.90), (x0 + ctx.rng.uniform(-0.05, 0.05), 0.95),
                   (x0 + ctx.rng.uniform(-0.08, 0.08), 0.99)], gray_d, 0.012)
    # hollow eyes
    for ex in (0.468, 0.532):
        ctx.ell(ex, 0.185, 0.012, 0.016, (104, 104, 112, 255))


def dark_reflection(ctx):
    gray, gray_d = (102, 102, 110, 255), (52, 52, 60, 255)
    humanoid(ctx, gray, dark=gray_d, hi=(128, 128, 138, 255), head=gray,
             leg_spread=0.12)
    for ex in (0.468, 0.532):  # eyes a shade too dark
        ctx.ell(ex, 0.185, 0.013, 0.017, (16, 16, 20, 255))


def twisted_reflection(ctx):
    gray, gray_d = (118, 118, 126, 255), (58, 58, 66, 255)
    humanoid(ctx, gray, dark=gray_d, hi=(142, 142, 150, 255), head=gray,
             arm_pose="long", distort=0.035)
    # unstable form: edges fray into short strokes
    for _ in range(12):
        x = ctx.rng.uniform(0.34, 0.66); y = ctx.rng.uniform(0.15, 0.9)
        ctx.line([(x, y), (x + ctx.rng.uniform(-0.04, 0.04), y + 0.03)], gray_d, 0.011)
    for ex in (0.462, 0.538):  # eyes not level
        ctx.ell(ex, 0.185 + (0.012 if ex > 0.5 else -0.006), 0.012, 0.016, (14, 14, 18, 255))


def better_half(ctx):
    # what you've dreamed you could be: idealized, upright, faintly radiant
    ctx.glow(0.5, 0.5, 0.52, (220, 214, 190), 60)
    body, hi = (128, 128, 136, 255), (164, 164, 172, 255)
    humanoid(ctx, body, dark=(70, 70, 78, 255), hi=hi, head=(206, 190, 168, 255),
             skin=(206, 190, 168, 255), leg_spread=0.07)
    for ex in (0.468, 0.532):  # calm, confident eyes
        ctx.ell(ex, 0.185, 0.011, 0.014, (30, 30, 36, 255))
    ctx.arc(0.5, 0.205, 0.030, 0.022, 20, 160, (90, 70, 60, 255), 0.010)  # slight smile


def blank_avatar_copy(ctx):
    # a smooth white changeling that is now you -- and smiles like you
    wht, wht_d = (240, 240, 238, 255), (168, 168, 164, 255)
    humanoid(ctx, wht, dark=wht_d, hi=(252, 252, 250, 255), head=wht, skin=wht)
    for ex in (0.468, 0.532):  # blank, too-even eyes
        ctx.ell(ex, 0.182, 0.010, 0.013, (120, 120, 118, 255))
    ctx.arc(0.5, 0.20, 0.034, 0.026, 15, 165, (120, 120, 118, 255), 0.013)  # the smile


def hallucinator(ctx):
    # a common human. exactly like a human looks. they move towards you.
    skin, shirt, pants = (216, 182, 148, 255), (196, 198, 204, 255), (96, 100, 116, 255)
    humanoid(ctx, shirt, dark=(70, 72, 80, 255), hi=(224, 226, 232, 255),
             head=skin, skin=skin)
    for sx in (-1, 1):  # trouser legs over the shared legs
        ctx.line([(0.5 + sx * 0.055, 0.56), (0.5 + sx * 0.10, 0.90)], pants, 0.055)
    for ex in (0.468, 0.532):  # eyes that know you can see them
        ctx.ell(ex, 0.185, 0.014, 0.017, (18, 16, 14, 255))
        ctx.ell(ex - 0.004, 0.180, 0.004, 0.004, (255, 255, 255, 255))


def xedra_officer(ctx):
    # high-tech suit viewed through a prism: red/cyan fringed outline
    for off, col in ((-0.016, (64, 200, 210, 255)), (0.016, (220, 60, 60, 255))):
        humanoid(ctx, col, dark=col, head=col, lean=0.03)
        # hide the fringe copy's interior detail under the real body next
    suit, suit_hi = (52, 56, 68, 255), (96, 106, 124, 255)
    humanoid(ctx, suit, dark=(22, 24, 30, 255), hi=suit_hi, head=(64, 70, 84, 255),
             skin=(64, 70, 84, 255), lean=0.03, arm_pose="reach")
    # suit lights and seams
    for lx, ly in ((0.44, 0.38), (0.56, 0.38), (0.50, 0.50)):
        ctx.ell(lx, ly, 0.010, 0.010, (120, 230, 240, 255))
    ctx.rect(0.455, 0.145, 0.545, 0.175, (150, 220, 228, 255), DARK)  # visor band


def tainted_shadow(ctx):
    # your shadow, host to something alien
    ctx.ell(0.5, 0.90, 0.34, 0.075, (30, 28, 38, 255))  # shadow pool
    ctx.ell(0.5, 0.90, 0.26, 0.05, (44, 40, 56, 255))
    body = (38, 34, 48, 255)
    humanoid(ctx, body, dark=(16, 14, 22, 255), head=body, arm_pose="long",
             leg_spread=0.13)
    for ex in (0.468, 0.532):  # alien glints where eyes should be
        ctx.ell(ex, 0.185, 0.011, 0.014, (150, 90, 200, 255))
    ctx.glow(0.5, 0.19, 0.06, (120, 60, 180), 50)


def highlands_revenant(ctx):
    cloth, cloth_d, cloth_hi = (206, 200, 188, 255), (148, 140, 126, 255), (230, 226, 214, 255)
    # too many limbs pressing against the shroud from inside
    for bx, by, br in ((0.40, 0.42, 0.05), (0.60, 0.50, 0.055), (0.44, 0.62, 0.045),
                       (0.58, 0.30, 0.04), (0.50, 0.74, 0.05)):
        ctx.ell(bx, by, br, br * 1.25, cloth_d)
    # the draped burial shroud itself
    shroud = [(0.40, 0.10), (0.60, 0.10), (0.66, 0.34), (0.64, 0.66),
              (0.68, 0.94), (0.32, 0.94), (0.36, 0.66), (0.34, 0.34)]
    ctx.poly(shroud, (120, 112, 100, 255))
    inner = [(0.5 + (x - 0.5) * 0.88, 0.5 + (y - 0.5) * 0.94) for x, y in shroud]
    ctx.poly(inner, cloth)
    ctx.poly([(0.43, 0.14), (0.50, 0.14), (0.485, 0.55), (0.445, 0.55)], cloth_hi)
    for fy in (0.36, 0.55, 0.74):  # cloth folds
        ctx.line([(0.40, fy), (0.60, fy + 0.02)], cloth_d, 0.012)
    # unnervingly life-like arms emerging, heavy with gold
    skin = (206, 172, 140, 255)
    for sx in (-1, 1):
        sh = (0.5 + sx * 0.155, 0.36)
        ha = (0.5 + sx * 0.235, 0.60)
        ctx.line([sh, (0.5 + sx * 0.22, 0.48), ha], (120, 112, 100, 255), 0.07)
        ctx.line([sh, (0.5 + sx * 0.22, 0.48), ha], skin, 0.048)
        ctx.ell(ha[0], ha[1], 0.030, 0.030, skin, cloth_d)
        for bx, by in ((0.5 + sx * 0.205, 0.47), (0.5 + sx * 0.225, 0.545)):  # bracelets
            ctx.ell(bx, by, 0.026, 0.014, None or (0, 0, 0, 0), (238, 196, 70, 255), 0.016)
        ctx.arc(0.5 + sx * 0.13, 0.40, 0.045, 0.05, 300, 120, (238, 196, 70, 255), 0.013)  # chain
    # shadowed face hollow
    ctx.ell(0.5, 0.185, 0.048, 0.058, (96, 90, 80, 255))


def dish_head(ctx):
    # emaciated little body under a massive bloated head with a dish for a face
    body = (104, 102, 108, 255)
    for sx in (-1, 1):  # spindly legs, mid measured step
        ctx.line([(0.5 + sx * 0.03, 0.80), (0.5 + sx * 0.07, 0.90),
                  (0.5 + sx * 0.05, 0.965)], DARK, 0.045)
        ctx.line([(0.5 + sx * 0.03, 0.80), (0.5 + sx * 0.07, 0.90),
                  (0.5 + sx * 0.05, 0.965)], body, 0.028)
        ctx.line([(0.5 + sx * 0.05, 0.68), (0.5 + sx * 0.13, 0.80)], DARK, 0.04)
        ctx.line([(0.5 + sx * 0.05, 0.68), (0.5 + sx * 0.13, 0.80)], body, 0.024)
    ctx.poly([(0.455, 0.64), (0.545, 0.64), (0.535, 0.82), (0.465, 0.82)], DARK)
    ctx.poly([(0.468, 0.655), (0.532, 0.655), (0.525, 0.805), (0.475, 0.805)], body)
    # the head
    ctx.ell(0.5, 0.335, 0.315, 0.275, DARK)
    ctx.ell(0.5, 0.335, 0.295, 0.255, (128, 126, 132, 255))
    ctx.ell(0.40, 0.20, 0.11, 0.07, (158, 156, 162, 255))  # bloated highlight
    # the dish divot, ringed with fang-like teeth
    for r, col in ((0.185, (96, 94, 102, 255)), (0.145, (74, 72, 82, 255)),
                   (0.105, (50, 48, 60, 255)), (0.06, (30, 28, 38, 255))):
        ctx.ell(0.5, 0.345, r, r * 0.88, col)
    for i in range(14):
        a = i * 2 * math.pi / 14
        tx = 0.5 + 0.20 * math.cos(a); ty = 0.345 + 0.176 * math.sin(a)
        ix = 0.5 + 0.168 * math.cos(a); iy = 0.345 + 0.148 * math.sin(a)
        ctx.poly([(tx - 0.012, ty), (tx + 0.012, ty), (ix, iy)], (236, 232, 220, 255))


def flatlander(ctx):
    # a paper-thin oblong figure floating parallel to the ground
    ctx.ell(0.5, 0.86, 0.30, 0.045, (120, 120, 128, 255))  # its shadow, far below
    layer = Image.new("RGBA", (ctx.size, ctx.size), (0, 0, 0, 0))
    ld = ImageDraw.Draw(layer)
    ld.ellipse([0.08 * ctx.size, 0.815 * ctx.size, 0.92 * ctx.size, 0.905 * ctx.size],
               fill=(90, 90, 100, 60))
    layer = layer.filter(ImageFilter.GaussianBlur(max(1, ctx.size // 16)))
    ctx.img = Image.alpha_composite(ctx.img, layer)
    ctx.d = ImageDraw.Draw(ctx.img)
    # the figure itself: an edge-on sliver with a hint of a head end
    ctx.poly([(0.10, 0.44), (0.86, 0.415), (0.90, 0.44), (0.86, 0.465), (0.10, 0.475)],
             (200, 200, 204, 255), DARK)
    ctx.line([(0.14, 0.452), (0.84, 0.435)], (244, 244, 246, 255), 0.012)
    ctx.ell(0.865, 0.44, 0.028, 0.020, (226, 226, 230, 255), DARK)  # head end
    ctx.line([(0.30, 0.475), (0.28, 0.53)], (170, 170, 176, 255), 0.010)  # trailing wisps
    ctx.line([(0.62, 0.462), (0.64, 0.52)], (170, 170, 176, 255), 0.010)


def shifting_blotch(ctx, droop=False):
    # guilt / sloth: an animate blotch of conflicting shapes
    tones = [(226, 226, 228, 255), (188, 188, 194, 255), (150, 150, 158, 255),
             (110, 110, 120, 255)]
    cy = 0.56 if droop else 0.50
    shapes = []
    for i in range(9):
        x = 0.5 + ctx.rng.uniform(-0.20, 0.20)
        y = cy + ctx.rng.uniform(-0.18, 0.18)
        r = ctx.rng.uniform(0.07, 0.15)
        shapes.append((x, y, r, i % 3))
    for x, y, r, kind in shapes:
        col = tones[ctx.rng.randrange(len(tones))]
        if kind == 0:
            ctx.ell(x, y, r, r * 0.85, col, DARK)
        elif kind == 1:
            ctx.poly([(x - r, y + r * 0.7), (x + r, y + r * 0.7), (x, y - r)], col, DARK)
        else:
            ctx.rect(x - r * 0.8, y - r * 0.6, x + r * 0.8, y + r * 0.6, col, DARK)
    # two hollow eye-holes that follow you
    for ex in (0.465, 0.535):
        ctx.ell(ex, cy - 0.05, 0.016, 0.022, (40, 38, 46, 255))


def guilt(ctx):
    shifting_blotch(ctx, droop=False)


def sloth(ctx):
    shifting_blotch(ctx, droop=True)


def carrion_grub(ctx):
    # wriggling segmented grub, S-curve, chewing head
    spine = []
    for i in range(8):
        t = i / 7.0
        x = 0.14 + 0.70 * t
        y = 0.62 - 0.16 * t + 0.13 * math.sin(t * 4.6)
        r = 0.075 + 0.05 * math.sin(t * math.pi) + (0.028 if i % 2 else 0)  # pulsation
        spine.append((x, y, r))
    browns = [(96, 64, 36, 255), (122, 84, 50, 255), (146, 104, 66, 255)]
    for i, (x, y, r) in enumerate(reversed(spine)):
        col = browns[i % 3]
        ctx.ell(x, y, r, r * 0.88, (62, 40, 22, 255))
        ri = r * 0.84
        ctx.ell(x, y, ri, ri * 0.84, col)
        ctx.ell(x - ri * 0.3, y - ri * 0.35, ri * 0.42, ri * 0.3, (172, 130, 88, 255))
    hx, hy, hr = spine[-1]  # head with flesh-chewing mandibles
    for sx in (-1, 1):
        ctx.curve([(hx + sx * 0.02, hy + 0.03), (hx + sx * 0.075, hy + 0.075),
                   (hx + sx * 0.045, hy + 0.115)], (52, 32, 16, 255), 0.02)
    ctx.ell(hx + 0.02, hy - 0.03, 0.016, 0.016, (30, 20, 12, 255))


def halo_tangle(ctx):
    # uncountable gleaming halos around a dark flame with a humanoid shadow
    gold, gold_hi, gold_d = (240, 196, 62, 255), (255, 226, 130, 255), (196, 148, 34, 255)
    ctx.glow(0.5, 0.5, 0.46, (240, 200, 80), 55)
    rings = [(0.50, 0.50, 0.40, 0.30), (0.46, 0.46, 0.34, 0.40), (0.55, 0.54, 0.38, 0.22),
             (0.50, 0.44, 0.28, 0.36), (0.44, 0.56, 0.30, 0.26), (0.56, 0.46, 0.24, 0.32),
             (0.50, 0.58, 0.34, 0.18), (0.42, 0.50, 0.20, 0.30), (0.58, 0.56, 0.22, 0.20),
             (0.50, 0.38, 0.18, 0.24)]
    for i, (cx, cy, rx, ry) in enumerate(rings):
        col = (gold, gold_hi, gold_d)[i % 3]
        a0 = (i * 47) % 360
        ctx.arc(cx, cy, rx, ry, a0, a0 + 300, col, 0.030)
    # dark flame core
    flame = [(0.5, 0.24), (0.575, 0.44), (0.60, 0.60), (0.545, 0.74),
             (0.455, 0.74), (0.40, 0.60), (0.425, 0.44)]
    ctx.poly(flame, (16, 12, 22, 255))
    ctx.poly([(0.5 + (x - 0.5) * 0.6, 0.5 + (y - 0.5) * 0.72) for x, y in flame],
             (34, 22, 44, 255))
    # twitching humanoid shadow inside the flame
    ctx.ell(0.5, 0.415, 0.030, 0.036, (6, 4, 10, 255))
    ctx.poly([(0.472, 0.45), (0.528, 0.45), (0.52, 0.60), (0.48, 0.60)], (6, 4, 10, 255))
    ctx.line([(0.475, 0.47), (0.435, 0.55)], (6, 4, 10, 255), 0.016)
    ctx.line([(0.525, 0.47), (0.57, 0.52)], (6, 4, 10, 255), 0.016)


def hanging_roper(ctx):
    # worm-like appendages hanging over a solid body that is mostly maw
    ctx.rect(0.10, 0.0, 0.90, 0.06, (60, 44, 48, 255), DARK)  # ceiling anchor
    pinks = [(196, 130, 140, 255), (172, 104, 116, 255), (216, 156, 164, 255)]
    for i in range(9):
        x0 = 0.16 + i * 0.085
        ln = ctx.rng.uniform(0.30, 0.52)
        ctx.curve([(x0, 0.05), (x0 + ctx.rng.uniform(-0.05, 0.05), ln * 0.55),
                   (x0 + ctx.rng.uniform(-0.07, 0.07), ln)], pinks[i % 3], 0.034)
    # solid body
    ctx.ell(0.5, 0.70, 0.30, 0.235, DARK)
    ctx.ell(0.5, 0.70, 0.275, 0.21, (150, 88, 96, 255))
    ctx.ell(0.40, 0.58, 0.10, 0.055, (196, 130, 140, 255))
    # the great maw: rows of serrated fangs
    ctx.ell(0.5, 0.72, 0.205, 0.145, (52, 16, 22, 255), DARK)
    for row, (rr, n, flip) in enumerate(((0.185, 9, 1), (0.145, 7, -1), (0.105, 5, 1))):
        for i in range(n):
            a = (i + 0.5) / n * 2 * math.pi
            tx = 0.5 + rr * math.cos(a); ty = 0.72 + rr * 0.70 * math.sin(a)
            ix = 0.5 + (rr - 0.035) * math.cos(a); iy = 0.72 + (rr - 0.035) * 0.70 * math.sin(a)
            if flip > 0:
                ctx.poly([(tx - 0.011, ty), (tx + 0.011, ty), (ix, iy)], (238, 234, 224, 255))
            else:
                ctx.poly([(tx, ty - 0.011), (tx, ty + 0.011), (ix, iy)], (226, 220, 208, 255))


def nether_cucumber(ctx):
    # pink translucent conical sea-creature; organs show through the skin
    skin, skin_d = (232, 156, 168, 255), (188, 104, 120, 255)
    # stubby furry legs
    for lx in (0.34, 0.44, 0.56, 0.66):
        ctx.ell(lx, 0.90, 0.045, 0.035, skin_d, DARK)
        for f in (-0.02, 0.0, 0.02):
            ctx.line([(lx + f, 0.925), (lx + f * 1.6, 0.955)], skin_d, 0.008)
    # conical body, point up
    cone = [(0.30, 0.88), (0.40, 0.42), (0.47, 0.16), (0.53, 0.16),
            (0.60, 0.42), (0.70, 0.88)]
    ctx.poly(cone, DARK)
    inner = [(0.5 + (x - 0.5) * 0.86, 0.52 + (y - 0.52) * 0.90) for x, y in cone]
    ctx.poly(inner, skin)
    # otherworldly organs visible through the skin
    for ox, oy, orx, ory, col in ((0.47, 0.36, 0.055, 0.075, (168, 84, 120, 255)),
                                  (0.55, 0.52, 0.065, 0.09, (140, 96, 160, 255)),
                                  (0.46, 0.66, 0.05, 0.065, (196, 120, 84, 255)),
                                  (0.53, 0.78, 0.045, 0.05, (120, 150, 110, 255))):
        ctx.ell(ox, oy, orx, ory, col)
    ctx.poly([(0.435, 0.20), (0.475, 0.20), (0.465, 0.55), (0.445, 0.55)],
             (244, 190, 198, 255))  # translucent sheen
    # rear orifice with barbed filaments (rear = the wide base, right side)
    ctx.ell(0.665, 0.82, 0.028, 0.036, (120, 60, 76, 255), DARK)
    for i in range(4):
        y0 = 0.76 + i * 0.035
        ctx.curve([(0.68, y0), (0.76, y0 + 0.02), (0.82, y0 - 0.015)], skin_d, 0.012)
        ctx.line([(0.755, y0 + 0.018), (0.745, y0 + 0.038)], skin_d, 0.008)  # barb


def nether_fish(ctx):
    # deformed eyeless piranha, swimming in yellow ichor
    for dx, dy in ((0.16, 0.30), (0.84, 0.24), (0.76, 0.78), (0.22, 0.80)):
        ctx.ell(dx, dy, 0.030, 0.038, (206, 172, 52, 255))  # ichor blobs
    body = [(0.10, 0.52), (0.30, 0.34), (0.58, 0.30), (0.80, 0.40),
            (0.92, 0.52), (0.80, 0.66), (0.55, 0.74), (0.28, 0.70), (0.12, 0.60)]
    ctx.poly(body, DARK)
    inner = [(0.5 + (x - 0.5) * 0.88, 0.52 + (y - 0.52) * 0.82) for x, y in body]
    ctx.poly(inner, (228, 208, 216, 255))
    ctx.poly([(0.30, 0.38), (0.55, 0.34), (0.52, 0.48), (0.32, 0.50)],
             (242, 232, 238, 255))  # sheen
    # tail + fins, slightly wrong
    ctx.poly([(0.12, 0.50), (0.03, 0.38), (0.05, 0.66)], (196, 160, 178, 255), DARK)
    ctx.poly([(0.48, 0.315), (0.60, 0.22), (0.66, 0.33)], (196, 160, 178, 255), DARK)
    ctx.poly([(0.50, 0.72), (0.62, 0.80), (0.64, 0.68)], (196, 160, 178, 255), DARK)
    # gaping jaw of jagged teeth; no eye -- the head is smooth
    ctx.poly([(0.78, 0.47), (0.925, 0.52), (0.80, 0.60), (0.72, 0.53)],
             (70, 26, 36, 255), DARK)
    for i in range(4):
        x = 0.79 + i * 0.032
        ctx.poly([(x, 0.485), (x + 0.014, 0.485), (x + 0.007, 0.525)], (240, 238, 230, 255))
        ctx.poly([(x, 0.585), (x + 0.014, 0.585), (x + 0.007, 0.55)], (240, 238, 230, 255))
    ctx.speckle(8, (0.3, 0.38, 0.7, 0.66), [(196, 90, 150, 255)], 0.005, 0.010)  # magenta flecks


def nether_spearfisher(ctx):
    pink, pink_d, hi = (206, 140, 150, 255), (156, 92, 102, 255), (228, 178, 184, 255)
    # long lanky quadruped legs
    for fx, fy in ((0.30, 0.95), (0.44, 0.90), (0.58, 0.90), (0.72, 0.95)):
        hip = (fx + (0.5 - fx) * 0.35, 0.55)
        knee = ((fx + hip[0]) / 2 + 0.03, 0.74)
        ctx.line([hip, knee, (fx, fy)], DARK, 0.055)
        ctx.line([hip, knee, (fx, fy)], pink_d, 0.036)
    # body
    ctx.ell(0.5, 0.50, 0.265, 0.155, DARK)
    ctx.ell(0.5, 0.50, 0.24, 0.13, pink)
    ctx.ell(0.42, 0.435, 0.10, 0.045, hi)
    # head with gaping maw
    ctx.ell(0.755, 0.47, 0.105, 0.09, DARK)
    ctx.ell(0.755, 0.47, 0.088, 0.073, pink)
    ctx.ell(0.775, 0.50, 0.058, 0.042, (70, 26, 34, 255))
    for i in range(3):
        x = 0.735 + i * 0.032
        ctx.poly([(x, 0.468), (x + 0.013, 0.468), (x + 0.006, 0.50)], (240, 238, 230, 255))
    ctx.ell(0.735, 0.425, 0.013, 0.013, (40, 26, 30, 255))  # eye
    # the back spike, launched up from its joint
    ctx.ell(0.44, 0.375, 0.035, 0.030, pink_d, DARK)  # joint
    ctx.poly([(0.415, 0.36), (0.465, 0.36), (0.44, 0.06)], (226, 222, 210, 255), DARK)
    ctx.line([(0.44, 0.33), (0.44, 0.10)], (244, 242, 234, 255), 0.010)


def unviable_bug(ctx):
    pink, pink_d = (216, 142, 156, 255), (168, 96, 108, 255)
    # three legs only
    for lx, ly in ((0.30, 0.88), (0.52, 0.92), (0.72, 0.86)):
        ctx.line([(lx + (0.5 - lx) * 0.3, 0.68), (lx, ly)], DARK, 0.04)
        ctx.line([(lx + (0.5 - lx) * 0.3, 0.68), (lx, ly)], pink_d, 0.024)
    # stink-bug shield body
    shield = [(0.32, 0.34), (0.68, 0.34), (0.74, 0.58), (0.62, 0.78),
              (0.38, 0.78), (0.26, 0.58)]
    ctx.poly(shield, DARK)
    inner = [(0.5 + (x - 0.5) * 0.86, 0.56 + (y - 0.56) * 0.86) for x, y in shield]
    ctx.poly(inner, pink)
    ctx.line([(0.5, 0.38), (0.5, 0.74)], pink_d, 0.016)  # shield seam
    ctx.poly([(0.40, 0.42), (0.47, 0.42), (0.44, 0.58), (0.40, 0.56)], (230, 178, 188, 255))
    # head + five antennae
    ctx.ell(0.5, 0.315, 0.075, 0.05, pink_d, DARK)
    for i, a in enumerate((-0.9, -0.45, 0.0, 0.45, 0.9)):
        tip = (0.5 + 0.16 * math.sin(a), 0.27 - 0.11 * math.cos(a) + 0.02 * abs(a))
        ctx.curve([(0.5, 0.29), (0.5 + 0.09 * math.sin(a), 0.20), tip], DARK, 0.013)
        ctx.ell(tip[0], tip[1], 0.010, 0.010, pink_d)


def unviable_butterfly(ctx):
    # barely there: a faint, translucent-seeming veiled butterfly, falling apart
    wing, wing_d, vein = (238, 196, 196, 255), (214, 156, 158, 255), (190, 128, 132, 255)
    def mirror(pts):
        return [(1.0 - x, y) for x, y in reversed(pts)]
    fore = [(0.515, 0.42), (0.66, 0.18), (0.84, 0.11), (0.955, 0.19),
            (0.96, 0.34), (0.87, 0.46), (0.66, 0.50), (0.53, 0.47)]
    hind = [(0.525, 0.53), (0.70, 0.51), (0.86, 0.61), (0.875, 0.77),
            (0.75, 0.88), (0.60, 0.83), (0.525, 0.65)]
    for w in (fore, hind):
        for side in (w, mirror(w)):
            ctx.poly(side, wing_d)
            cx = sum(x for x, _ in side) / len(side); cy = sum(y for _, y in side) / len(side)
            inner = [(cx + (x - cx) * 0.84, cy + (y - cy) * 0.82) for x, y in side]
            ctx.poly(inner, wing)
            root = (0.53, 0.45) if w is fore else (0.535, 0.56)
            if side is not w:
                root = (1.0 - root[0], root[1])
            for vx, vy in side[1:4]:
                if side is not w:
                    vx = 1.0 - vx
                ctx.line([root, (vx, vy)], vein, 0.010)
    # veil: a few soft strands over the wings (opaque pale lines)
    for vx in (0.24, 0.5, 0.76):
        ctx.curve([(vx, 0.10), (vx + 0.03, 0.4), (vx - 0.02, 0.72)],
                  (246, 226, 224, 255), 0.010)
    # thin body + falling-apart flecks
    ctx.ell(0.5, 0.40, 0.030, 0.075, (150, 104, 108, 255))
    ctx.ell(0.5, 0.62, 0.024, 0.14, (150, 104, 108, 255))
    ctx.ell(0.5, 0.29, 0.026, 0.026, (150, 104, 108, 255))
    for fx, fy in ((0.16, 0.86), (0.86, 0.82), (0.70, 0.93), (0.30, 0.95)):
        ctx.ell(fx, fy, 0.012, 0.008, wing_d)


def unviable_root(ctx):
    # a chunk of roots, writhing and failing to grow
    tones = [(172, 170, 164, 255), (140, 138, 130, 255), (110, 108, 100, 255)]
    ctx.ell(0.5, 0.44, 0.16, 0.13, tones[2])  # central knot
    ctx.ell(0.5, 0.44, 0.13, 0.10, tones[1])
    ctx.ell(0.455, 0.395, 0.05, 0.035, tones[0])
    for i in range(11):
        a = i / 11 * 2 * math.pi + ctx.rng.uniform(-0.2, 0.2)
        x0 = 0.5 + 0.10 * math.cos(a); y0 = 0.44 + 0.08 * math.sin(a)
        ln = ctx.rng.uniform(0.16, 0.30)
        pts = [(x0, y0),
               (x0 + ln * 0.5 * math.cos(a + 0.5), y0 + ln * 0.5 * math.sin(a + 0.5)),
               (x0 + ln * math.cos(a + ctx.rng.uniform(-0.4, 0.4)),
                y0 + ln * math.sin(a + ctx.rng.uniform(-0.4, 0.4)))]
        ctx.curve(pts, tones[i % 3], 0.026 if i % 2 else 0.034)
        ctx.ell(pts[-1][0], pts[-1][1], 0.012, 0.012, tones[0])  # failed growth tip
    for bx, by in ((0.44, 0.30), (0.58, 0.36), (0.50, 0.55)):  # pale buds
        ctx.ell(bx, by, 0.016, 0.020, (196, 200, 178, 255), tones[2])


def unviable_stomper(ctx):
    blue, blue_d, hair = (86, 110, 178, 255), (56, 76, 128, 255), (70, 92, 156, 255)
    # the single pronounced leg
    ctx.line([(0.52, 0.55), (0.50, 0.80), (0.52, 0.93)], DARK, 0.13)
    ctx.line([(0.52, 0.55), (0.50, 0.80), (0.52, 0.93)], blue_d, 0.09)
    ctx.ell(0.55, 0.945, 0.075, 0.028, blue_d, DARK)  # foot
    # body mass
    ctx.ell(0.47, 0.40, 0.27, 0.235, DARK)
    ctx.ell(0.47, 0.40, 0.245, 0.21, blue)
    ctx.ell(0.39, 0.30, 0.09, 0.06, (120, 142, 204, 255))
    # long wet hair strands hanging everywhere, massed at the rear
    for i in range(16):
        x0 = 0.26 + i * 0.028 + ctx.rng.uniform(-0.008, 0.008)
        rear = x0 < 0.38
        ln = ctx.rng.uniform(0.30, 0.44) if rear else ctx.rng.uniform(0.18, 0.32)
        ctx.curve([(x0, 0.30 + ctx.rng.uniform(0, 0.08)),
                   (x0 - 0.03, 0.30 + ln * 0.55), (x0 - 0.05, 0.30 + ln)],
                  hair if i % 2 else blue_d, 0.016)
    ctx.ell(0.60, 0.36, 0.016, 0.020, (16, 20, 34, 255))  # eye in the hair


def shrapnel_swarm(ctx):
    # uncountable jagged spikes cleaving through reality's seams
    dark = (86, 86, 94, 255)
    for _ in range(16):
        x = ctx.rng.uniform(0.14, 0.86); y = ctx.rng.uniform(0.14, 0.86)
        ln = ctx.rng.uniform(0.07, 0.16); wd = ctx.rng.uniform(0.016, 0.032)
        tip = (x + ln * 0.8, y - ln * 0.45)
        ctx.poly([(x, y), (x + wd, y + wd * 0.6), tip], dark, DARK)
        ctx.line([(x - ln * 0.35, y + ln * 0.2), (x, y)], (150, 150, 158, 255), 0.010)
    for sx, sy in ((0.30, 0.34), (0.62, 0.58), (0.48, 0.76)):  # seam glints
        ctx.line([(sx - 0.03, sy), (sx + 0.03, sy)], (230, 230, 236, 255), 0.010)
        ctx.line([(sx, sy - 0.03), (sx, sy + 0.03)], (230, 230, 236, 255), 0.010)


def structural_spur(ctx):
    # misshapen concrete-and-rebar spikes crumbling upward, held by distortion
    conc, conc_d = (208, 204, 200, 255), (156, 150, 146, 255)
    spikes = [(0.16, 0.42, 0.30), (0.34, 0.22, 0.24), (0.52, 0.34, 0.28),
              (0.68, 0.16, 0.22), (0.84, 0.40, 0.26)]
    for cx, top, wd in spikes:
        pts = [(cx - wd / 2, 0.97), (cx - wd * 0.18, top + 0.10), (cx, top),
               (cx + wd * 0.22, top + 0.08), (cx + wd / 2, 0.97)]
        ctx.poly(pts, conc_d)
        inner = [(cx + (x - cx) * 0.78, 0.97 - (0.97 - y) * 0.92) for x, y in pts]
        ctx.poly(inner, conc)
        ctx.line([(cx, top - 0.02), (cx + 0.015, top - 0.10)], (84, 80, 78, 255), 0.016)  # rebar
        ctx.line([(cx - 0.03, top + 0.03), (cx - 0.045, top - 0.05)], (84, 80, 78, 255), 0.013)
    ctx.rect(0.06, 0.90, 0.94, 0.985, conc_d, DARK)  # rubble base
    ctx.speckle(12, (0.1, 0.88, 0.9, 0.96), [conc, (240, 120, 110, 255)], 0.006, 0.013)
    for wx in (0.26, 0.60, 0.78):  # strands of visual distortion
        pts = [(wx + 0.02 * math.sin(y * 26), y) for y in (0.12, 0.3, 0.48, 0.66, 0.84)]
        ctx.line(pts, (244, 170, 160, 255), 0.010)


def swarm_structure(ctx):
    # is it a vast structure, or an endless swarm of tiny sharp things?
    dark, mid = (74, 74, 82, 255), (110, 110, 120, 255)
    # blocky structure on the left, too large for its space
    ctx.rect(0.06, 0.22, 0.52, 0.90, DARK)
    ctx.rect(0.085, 0.245, 0.495, 0.875, dark)
    ctx.rect(0.14, 0.16, 0.44, 0.26, mid, DARK)  # cap slab overhanging
    for wy in (0.36, 0.52, 0.68):  # slit windows
        ctx.rect(0.14, wy, 0.42, wy + 0.045, (30, 30, 38, 255))
        ctx.line([(0.14, wy + 0.022), (0.42, wy + 0.022)], (200, 200, 120, 255), 0.008)
    # dissolving into the swarm on the right
    for _ in range(42):
        x = ctx.rng.uniform(0.50, 0.95); y = ctx.rng.uniform(0.14, 0.90)
        pull = (x - 0.50) / 0.45
        y = y * (1 - pull * 0.3) + pull * ctx.rng.uniform(0.1, 0.5)
        r = ctx.rng.uniform(0.010, 0.030) * (1.2 - pull * 0.5)
        ctx.poly([(x, y - r), (x + r, y + r), (x - r, y + r)], mid if _ % 2 else dark, DARK)
    ctx.speckle(16, (0.5, 0.2, 0.9, 0.85), [(200, 200, 208, 255)], 0.004, 0.008)


def twisting_mass(ctx):
    # a dense, suffocating mass of fluorescent purple strings, lashing out
    ctx.glow(0.5, 0.5, 0.48, (170, 50, 220), 60)
    tones = [(216, 70, 255, 255), (178, 44, 226, 255), (240, 130, 255, 255),
             (146, 30, 190, 255)]
    for i in range(30):
        y0 = 0.14 + (i % 10) * 0.075 + ctx.rng.uniform(-0.015, 0.015)
        x0 = 0.16 + (i // 10) * 0.10
        pts = []
        for k in range(5):
            t = k / 4
            pts.append((x0 + 0.55 * t, y0 + 0.10 * math.sin(t * 6 + i * 1.7) * (1 - t * 0.4)))
        ctx.curve(pts, tones[i % 4], 0.016)
    # near-solid core
    ctx.ell(0.46, 0.50, 0.20, 0.24, (120, 26, 160, 255))
    ctx.ell(0.46, 0.50, 0.15, 0.18, (160, 40, 205, 255))
    for i in range(10):  # core strings still writhing
        y0 = 0.34 + i * 0.032
        pts = [(0.33 + 0.26 * t, y0 + 0.03 * math.sin(t * 9 + i)) for t in (0, 0.33, 0.66, 1.0)]
        ctx.curve(pts, tones[(i + 1) % 4], 0.013)
    # individual strings lashing far out
    for a in (0.4, 1.5, 2.6, 3.6, 4.7, 5.8):
        tip = (0.46 + 0.52 * math.cos(a), 0.50 + 0.52 * math.sin(a))
        mid = (0.46 + 0.33 * math.cos(a + 0.4), 0.50 + 0.33 * math.sin(a + 0.4))
        ctx.curve([(0.46, 0.50), mid, tip], tones[int(a) % 4], 0.014)


def unseen_hunter(ctx):
    # the faintest outline of an otherworldly creature, beak and bladed legs
    faint = (196, 220, 232, 255)
    edge = (158, 190, 208, 255)
    # rows upon rows of bladed legs
    for i in range(7):
        x = 0.24 + i * 0.085
        ctx.poly([(x, 0.66), (x + 0.055, 0.66), (x + 0.028, 0.86)], faint, edge)
        ctx.poly([(x + 0.02, 0.70), (x + 0.075, 0.70), (x + 0.048, 0.88)], faint, edge)
    # body: only an outline, barely filled
    body = [(0.22, 0.52), (0.34, 0.34), (0.58, 0.28), (0.78, 0.38),
            (0.80, 0.56), (0.62, 0.68), (0.34, 0.68), (0.22, 0.60)]
    ctx.poly(body, (216, 234, 242, 255))
    ctx.poly(body, None, edge) if False else None
    ctx.d.polygon(ctx.X(body), outline=edge, width=ctx.w(0.016))
    ctx.curve([(0.30, 0.40), (0.48, 0.33), (0.66, 0.40)], edge, 0.012)
    # pointed beak, dripping translucent fluid
    ctx.poly([(0.76, 0.40), (0.95, 0.485), (0.765, 0.53)], faint, edge)
    for dx in (0.90, 0.945):
        ctx.line([(dx, 0.50), (dx, 0.565)], (170, 226, 236, 255), 0.011)
        ctx.ell(dx, 0.585, 0.011, 0.015, (170, 226, 236, 255))
    ctx.ell(0.70, 0.42, 0.013, 0.013, edge)  # one almost-eye


# ------------------------------------------------- star vampires / void ---

def star_vampire(ctx, coiling=False):
    # a small central body that is mostly mouth, tentacles tipped with maws
    tones = [(196, 60, 168, 255), (226, 100, 190, 255), (168, 40, 142, 255)] \
        if not coiling else [(226, 130, 160, 255), (206, 104, 132, 255), (240, 164, 184, 255)]
    cx, cy = 0.5, 0.5
    n = 14 if coiling else 22
    for i in range(n):
        a = i / n * 2 * math.pi + ctx.rng.uniform(-0.08, 0.08)
        reach = ctx.rng.uniform(0.36, 0.47)
        tip = (cx + reach * math.cos(a), cy + reach * math.sin(a))
        if coiling:  # thick tentacles coil in spirals
            mid1 = (cx + reach * 0.4 * math.cos(a + 0.55), cy + reach * 0.4 * math.sin(a + 0.55))
            mid2 = (cx + reach * 0.75 * math.cos(a - 0.35), cy + reach * 0.75 * math.sin(a - 0.35))
            ctx.curve([(cx, cy), mid1, mid2, tip], DARK, 0.052)
            ctx.curve([(cx, cy), mid1, mid2, tip], tones[i % 3], 0.036)
            maw_r = 0.036
        else:  # myriad thin tentacles
            mid = (cx + reach * 0.55 * math.cos(a + 0.25), cy + reach * 0.55 * math.sin(a + 0.25))
            ctx.curve([(cx, cy), mid, tip], tones[i % 3], 0.016)
            maw_r = 0.024
        # the gnashing maw at the tip (flat maws on the coiling form)
        ctx.ell(tip[0], tip[1], maw_r * (1.35 if coiling else 1.0), maw_r * 0.8,
                (88, 20, 40, 255), DARK)
        for f in (-0.4, 0.0, 0.4):
            tx = tip[0] + f * maw_r
            ctx.poly([(tx - 0.007, tip[1] - maw_r * 0.55), (tx + 0.007, tip[1] - maw_r * 0.55),
                      (tx, tip[1] - maw_r * 0.05)], (240, 236, 226, 255))
    # small central body dominated by an enormous fanged mouth
    ctx.ell(cx, cy, 0.115, 0.105, DARK)
    ctx.ell(cx, cy, 0.098, 0.088, tones[1])
    ctx.ell(cx, cy + 0.008, 0.072, 0.058, (70, 14, 30, 255), DARK)
    for i in range(6):
        x = cx - 0.055 + i * 0.022
        ctx.poly([(x, cy - 0.028), (x + 0.016, cy - 0.028), (x + 0.008, cy + 0.012)],
                 (242, 238, 228, 255))
        ctx.poly([(x, cy + 0.048), (x + 0.016, cy + 0.048), (x + 0.008, cy + 0.010)],
                 (242, 238, 228, 255))
    ctx.ell(cx - 0.03, cy - 0.045, 0.018, 0.014, (250, 190, 220, 255))  # body sheen


def star_vampire_main(ctx):
    star_vampire(ctx, coiling=False)


def star_vampire_coiling(ctx):
    star_vampire(ctx, coiling=True)


def void_spider(ctx):
    # the unformed weaver: gossamer threads of reality around a kaleidoscopic maw
    for gx, gy in ((0.06, 0.06), (0.94, 0.06), (0.06, 0.94), (0.94, 0.94),
                   (0.5, 0.03), (0.5, 0.97), (0.03, 0.5), (0.97, 0.5)):
        ctx.line([(gx, gy), (0.5, 0.5)], (206, 210, 220, 255), 0.008)
    for _ in range(14):  # more threads, slightly astray
        a = ctx.rng.uniform(0, 2 * math.pi)
        ctx.line([(0.5, 0.5), (0.5 + 0.48 * math.cos(a), 0.5 + 0.48 * math.sin(a))],
                 (176, 182, 196, 255), 0.006)
    # long shadow legs, bent around the maw
    for i, a in enumerate((0.35, 1.15, 1.95, 2.75, 3.55, 4.35, 5.15, 5.95)):
        knee = (0.5 + 0.30 * math.cos(a), 0.5 + 0.30 * math.sin(a))
        foot = (0.5 + 0.46 * math.cos(a + 0.30), 0.5 + 0.46 * math.sin(a + 0.30))
        ctx.line([(0.5, 0.5), knee, foot], (18, 16, 26, 255), 0.030)
    # kaleidoscopic maw: concentric rotated stars
    cols = [(64, 200, 190, 255), (140, 90, 220, 255), (226, 70, 160, 255),
            (240, 180, 70, 255), (80, 140, 235, 255)]
    for ring, (r, col) in enumerate(((0.185, cols[0]), (0.148, cols[1]), (0.112, cols[2]),
                                     (0.078, cols[3]), (0.046, cols[4]))):
        pts = []
        for i in range(16):
            a = i * math.pi / 8 + ring * 0.20
            rr = r if i % 2 == 0 else r * 0.62
            pts.append((0.5 + rr * math.cos(a), 0.5 + rr * math.sin(a)))
        ctx.poly(pts, col, DARK)
    ctx.ell(0.5, 0.5, 0.020, 0.020, (10, 8, 14, 255))
    for i in range(10):  # glistening fangs around the inner maw
        a = i * math.pi / 5 + 0.3
        bx = 0.5 + 0.052 * math.cos(a); by = 0.5 + 0.052 * math.sin(a)
        tx = 0.5 + 0.030 * math.cos(a); ty = 0.5 + 0.030 * math.sin(a)
        ctx.poly([(bx - 0.008, by), (bx + 0.008, by), (tx, ty)], (240, 240, 234, 255))


def void_spider_limb(ctx):
    # strands of shadow snapping from their ordained positions
    ctx.rect(0.0, 0.0, 1.0, 0.16, (22, 20, 30, 255))  # darkness above
    ctx.rect(0.0, 0.86, 1.0, 1.0, (22, 20, 30, 255))  # and below
    for x0, bend in ((0.16, 1), (0.38, -1), (0.62, 1), (0.84, -1)):
        tip = (x0 + bend * 0.10, 0.52)
        ctx.curve([(x0, 0.10), (x0 + bend * 0.03, 0.30), tip], (16, 14, 24, 255), 0.055)
        ctx.curve([(x0 + 0.03, 0.90), (x0 + 0.05, 0.72), (tip[0] + 0.02, 0.60)],
                  (16, 14, 24, 255), 0.042)
        # snapping jagged tip
        ctx.poly([(tip[0] - 0.02, tip[1]), (tip[0] + 0.025, tip[1] - 0.015),
                  (tip[0] + 0.005, tip[1] + 0.035)], (16, 14, 24, 255))
    for _ in range(10):  # faint threads they snapped from
        x = ctx.rng.uniform(0.08, 0.92)
        ctx.line([(x, 0.16), (x + ctx.rng.uniform(-0.03, 0.03), 0.30)],
                 (150, 154, 168, 255), 0.006)
    ctx.glow(0.5, 0.52, 0.10, (90, 80, 130), 45)


def void_spider_maw(ctx):
    # an explosion of glistening fangs, a fragment of a greater horror
    ctx.ell(0.5, 0.5, 0.40, 0.38, (30, 16, 26, 255))  # the dark of the mouth
    ctx.ell(0.5, 0.5, 0.30, 0.28, (52, 20, 36, 255))
    for i in range(26):  # fangs radiating inward, all sizes
        a = i / 26 * 2 * math.pi + ctx.rng.uniform(-0.06, 0.06)
        r_out = ctx.rng.uniform(0.30, 0.42)
        ln = ctx.rng.uniform(0.10, 0.22)
        bx = 0.5 + r_out * math.cos(a); by = 0.5 + r_out * math.sin(a)
        tx = 0.5 + (r_out - ln) * math.cos(a); ty = 0.5 + (r_out - ln) * math.sin(a)
        px, py = -math.sin(a) * 0.016, math.cos(a) * 0.016
        ctx.poly([(bx - px, by - py), (bx + px, by + py), (tx, ty)],
                 (238, 236, 228, 255), (170, 166, 158, 255))
        ctx.line([(bx, by), (tx, ty)], (255, 252, 244, 255), 0.006)  # glisten
    ctx.ell(0.5, 0.5, 0.085, 0.075, (16, 8, 14, 255))  # throat
    for sx, sy in ((0.30, 0.24), (0.68, 0.30), (0.40, 0.74)):  # saliva glints
        ctx.ell(sx, sy, 0.012, 0.008, (220, 230, 235, 255))


def void_spider_spiderling(ctx, skewered=False):
    # a juvenile: conjoined layers of flailing legs and fangs
    red, red_d = (188, 52, 48, 255), (128, 32, 34, 255)
    cx, cy = 0.5, 0.52
    for i in range(12):  # flailing legs, two layers
        a = i / 12 * 2 * math.pi + (0.13 if i % 2 else 0)
        knee = (cx + 0.17 * math.cos(a), cy + 0.17 * math.sin(a))
        foot = (cx + ctx.rng.uniform(0.24, 0.30) * math.cos(a + 0.35),
                cy + ctx.rng.uniform(0.24, 0.30) * math.sin(a + 0.35))
        ctx.line([(cx, cy), knee, foot], (24, 20, 28, 255), 0.026)
        ctx.line([(cx, cy), knee, foot], red_d if i % 2 else (60, 40, 44, 255), 0.016)
    ctx.ell(cx, cy, 0.145, 0.125, (24, 20, 28, 255))
    ctx.ell(cx, cy, 0.125, 0.105, red)
    ctx.ell(cx - 0.04, cy - 0.045, 0.05, 0.035, (222, 96, 88, 255))
    for i in range(8):  # fang cluster at the front
        a = -0.7 + i * 0.2
        bx = cx + 0.10 * math.cos(a); by = cy + 0.085 * math.sin(a) + 0.03
        tx = cx + 0.155 * math.cos(a); ty = cy + 0.13 * math.sin(a) + 0.045
        ctx.poly([(bx - 0.009, by), (bx + 0.009, by), (tx, ty)], (240, 238, 230, 255))
    for ex in (0.455, 0.545):  # distressed eyes
        ctx.ell(ex, cy - 0.055, 0.013, 0.013, (20, 10, 12, 255))
    if skewered:
        # impaled on an emerald lance, limbs still struggling
        ctx.line([(0.08, 0.94), (0.94, 0.08)], (30, 90, 54, 255), 0.045)
        ctx.line([(0.08, 0.94), (0.94, 0.08)], (52, 178, 100, 255), 0.028)
        ctx.line([(0.30, 0.70), (0.72, 0.28)], (150, 240, 180, 255), 0.010)
        ctx.poly([(0.94, 0.08), (0.875, 0.115), (0.905, 0.175)], (120, 230, 160, 255), DARK)
        ctx.ell(0.44, 0.60, 0.020, 0.013, (120, 40, 40, 255))  # ichor at the wound


def spiderling(ctx):
    void_spider_spiderling(ctx, skewered=False)


def spiderling_skewered(ctx):
    void_spider_spiderling(ctx, skewered=True)


# ------------------------------------------------------------------ jobs ---

S32 = "pngs_normal_32x32/monsters"
S64 = "pngs_large_64x64/monsters"
S96 = "pngs_giant_96x96/monsters"

JOBS = [
    # labyrinth robots and constructs
    (S32, "mon_LS_fixbot", 32, fixbot),
    (S32, "mon_LS_fixbot_haywire", 32, fixbot_haywire),
    (S32, "mon_LS_medbot", 32, medbot),
    (S32, "mon_LS_spotbot", 32, spotbot),
    (S96, "mon_LS_robot_arm", 96, robot_arm),
    (S32, "mon_LS_sparkgenerator", 32, sparkgenerator),
    (S32, "mon_LS_sparkgenerator_aux", 32, sparkgenerator_aux),
    (S32, "mon_LS_sparkgenerator_aux_haywire", 32, sparkgenerator_aux_haywire),
    (S32, "mon_LS_sparkgenerator_haywire", 32, sparkgenerator_haywire),
    (S32, "mon_nl_surgicalbot_1_1", 32, surgicalbot_1_1),
    (S32, "mon_nl_surgicalbot_assist", 32, surgicalbot_assist),
    (S64, "mon_overgrown_labyrinth_turret_2_1", 64, vine_turret),
    (S64, "mon_plantborg_2_1", 64, plantborg_2_1),
    (S64, "mon_plantborg_2_2", 64, plantborg_2_2),
    (S64, "mon_vinewalker_2_1", 64, vinewalker),
    (S96, "mon_wells_machine", 96, wells_machine),
    (S96, "mon_wells_machine_leg", 96, wells_machine_leg),
    # nether humanoids and reflections
    (S32, "mon_XEDRA_officer", 32, xedra_officer),
    (S32, "mon_pale_reflection", 32, pale_reflection),
    (S32, "mon_dark_reflection", 32, dark_reflection),
    (S32, "mon_twisted_reflection", 32, twisted_reflection),
    (S32, "mon_better_half", 32, better_half),
    (S32, "mon_blank_avatar_copy", 32, blank_avatar_copy),
    (S32, "mon_hallucinator", 32, hallucinator),
    (S32, "mon_tainted_shadow", 32, tainted_shadow),
    (S32, "mon_highlands_revenant", 32, highlands_revenant),
    (S32, "mon_dish_head", 32, dish_head),
    (S32, "mon_flatlander", 32, flatlander),
    (S32, "mon_guilt", 32, guilt),
    (S32, "mon_sloth", 32, sloth),
    # nether creatures
    (S32, "mon_carrion_grub", 32, carrion_grub),
    (S64, "mon_halo_tangle", 64, halo_tangle),
    (S32, "mon_hanging_roper", 32, hanging_roper),
    (S32, "mon_nether_cucumber", 32, nether_cucumber),
    (S32, "mon_nether_fish", 32, nether_fish),
    (S32, "mon_nether_spearfisher", 32, nether_spearfisher),
    (S32, "mon_nether_unviable_bug", 32, unviable_bug),
    (S32, "mon_nether_unviable_butterfly", 32, unviable_butterfly),
    (S32, "mon_nether_unviable_root", 32, unviable_root),
    (S32, "mon_nether_unviable_stomper", 32, unviable_stomper),
    (S32, "mon_shrapnel_swarm", 32, shrapnel_swarm),
    (S32, "mon_structural_spur", 32, structural_spur),
    (S32, "mon_swarm_structure", 32, swarm_structure),
    (S64, "mon_twisting_mass", 64, twisting_mass),
    (S32, "mon_unseen_hunter", 32, unseen_hunter),
    # star vampires and the void spider
    (S96, "mon_star_vampire", 96, star_vampire_main),
    (S96, "mon_star_vampire_coiling", 96, star_vampire_coiling),
    (S96, "mon_void_spider", 96, void_spider),
    (S96, "mon_void_spider_limb", 96, void_spider_limb),
    (S96, "mon_void_spider_maw", 96, void_spider_maw),
    (S96, "mon_void_spider_spiderling", 96, spiderling),
    (S96, "mon_void_spider_spiderling_skewered", 96, spiderling_skewered),
]

for sheet, mid, size, fn in JOBS:
    folder = os.path.join(BASE, sheet, mid)
    os.makedirs(folder, exist_ok=True)
    ctx = Ctx(size, mid)
    fn(ctx)
    ctx.img.save(os.path.join(folder, mid + ".png"))
    with open(os.path.join(folder, mid + ".json"), "w") as f:
        json.dump([{"id": mid, "fg": mid, "bg": ""}], f, indent=2)
        f.write("\n")
    with open(os.path.join(folder, "credits.txt"), "w") as f:
        f.write(CREDITS)
    print("wrote", sheet + "/" + mid, ctx.img.size)
print("total", len(JOBS))
