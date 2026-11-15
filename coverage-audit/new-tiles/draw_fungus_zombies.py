#!/usr/bin/env python3
"""
Draw the missing fungal-zombie sprites from their game descriptions.

Sources (fork-src/cdda/data/json/monsters/fungus_zombie.json, CDDA 6a3e59e0):
50 entries, all variants of the fungal-infection zombie line:

  *_dusted    "...has been heavily dusted with a thick gray powder."
  *_fungalize "...has grown a short stalk of gray fungus bursting from
              the skull."
  *_fungus / *_fungal (skeletal pair, brute, hulk) -- full bloom: "Fungi
              bloom from the crevices... Spores linger in the air around it."
              / "chest and face covered in thick fungal sprouts."

Base bodies: plain zombie, smoker (smoke-shrouded), skeletal (knobby bone
growths; brute = twisted growths, hulk = plated bone tower), child, wretch
(prone, chest burst open "like a gory flower"), spawn raptor (small winged,
three hand-like wings + jagged spike), runner, rot (badly decayed), tough
(muscular), brute (distended muscles), hulk (six times human size), crawler
(damaged legs), fat, scarred (beekeeper), acidic (yellow veins), boomer
(rotund, magenta-tinged), gasbag (grotesquely swollen, black/yellow; the
fungalize one reads as "a cherry or upside down lollipop"), fungal
mound/construct (pale piles of fungus over gore; constructs have interwoven
vines/roots forcing the pulp to move), zougar (fungal pouncer cat, spores
spewing from pawpads), ziger (tiger wight, fungal bulbs extending pawpads).

Design: one parameterized front-facing shambling humanoid renderer (plus
dedicated prone/wretch, raptor, mound, and quadruped renderers). Sheet
placement follows the established volume precedent of the butterfly batch
(15 L -> 32, ~70 L -> 64, 120 L -> 96): standard 62.5 L humanoids at 32,
bulky 100-150 L bodies at 64, 875 L hulks and massive mounds at 96.
Dusting is a powder wash + speckle composited OVER the finished body via
Image.alpha_composite (never semi-transparent body pixels); spore motes are
small opaque dots. Dark outline, flat fills + one highlight pass -- UltiCa
monster style. Output is UltiCa decomposed layout (png + tile-entry json
per folder). This script only ever creates/overwrites its own 50 folders.
"""
from PIL import Image, ImageDraw
import json, os, random, zlib

BASE = os.path.dirname(os.path.abspath(__file__))

OUTLINE = (24, 26, 22, 255)
SKIN = (120, 136, 104, 255); SKIN_HI = (150, 164, 128, 255); SKIN_DK = (90, 104, 78, 255)
SHIRT = (88, 79, 66, 255); SHIRT_DK = (64, 57, 47, 255); PANTS = (56, 57, 66, 255)
GORE = (122, 42, 37, 255); GORE_DK = (82, 28, 26, 255)
BONE = (216, 208, 188, 255); BONE_HI = (236, 230, 212, 255); BONE_DK = (166, 156, 136, 255)
STALK = (188, 188, 178, 255); CAP = (148, 146, 136, 255); CAP_HI = (208, 206, 194, 255)
SPROUT = (172, 174, 160, 255); SPORE = (226, 224, 210, 255)
POWDER = (178, 178, 172, 255); POWDER_HI = (198, 198, 190, 255)

def rng_for(name):
    return random.Random(zlib.crc32(name.encode()))

class Ctx:
    def __init__(self, size):
        self.size = size
        self.img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
        self.d = ImageDraw.Draw(self.img)
    def w(self, frac=1.0 / 32):
        return max(1, round(self.size * frac))
    def E(self, box, **kw):
        s = self.size
        self.d.ellipse([box[0] * s, box[1] * s, box[2] * s, box[3] * s], **kw)
    def P(self, pts, **kw):
        s = self.size
        self.d.polygon([(x * s, y * s) for x, y in pts], **kw)
    def L(self, pts, fill, width=None):
        s = self.size
        self.d.line([(x * s, y * s) for x, y in pts], fill=fill,
                    width=width if width else self.w())
    def R(self, box, **kw):
        s = self.size
        self.d.rectangle([box[0] * s, box[1] * s, box[2] * s, box[3] * s], **kw)

def stalk(ctx, x, y, h=0.11, lean=0.0, cap_r=0.030):
    """A short gray fungal stalk with a cap, rooted at (x, y), growing up."""
    d = ctx.d; s = ctx.size
    top_x, top_y = x + lean, y - h
    ctx.L([(x, y), ((x + top_x) / 2 + lean * 0.3, (y + top_y) / 2), (top_x, top_y)],
          OUTLINE, ctx.w(0.030))
    ctx.L([(x, y), ((x + top_x) / 2 + lean * 0.3, (y + top_y) / 2), (top_x, top_y)],
          STALK, ctx.w(0.016))
    ctx.E([top_x - cap_r, top_y - cap_r * 0.75, top_x + cap_r, top_y + cap_r * 0.75],
          fill=CAP, outline=OUTLINE, width=ctx.w(0.012))
    ctx.E([top_x - cap_r * 0.5, top_y - cap_r * 0.55, top_x + cap_r * 0.1, top_y],
          fill=CAP_HI)

def sprout_cluster(ctx, cx, cy, r, rng, n=7):
    for _ in range(n):
        x = cx + rng.uniform(-r, r); y = cy + rng.uniform(-r * 0.7, r * 0.7)
        rr = rng.uniform(0.006, 0.016)
        ctx.E([x - rr, y - rr, x + rr, y + rr], fill=SPROUT, outline=OUTLINE,
              width=ctx.w(0.008))

def spore_motes(ctx, rng, n=10, region=(0.06, 0.06, 0.94, 0.94)):
    for _ in range(n):
        x = rng.uniform(region[0], region[2]); y = rng.uniform(region[1], region[3])
        rr = rng.uniform(0.005, 0.011)
        ctx.E([x - rr, y - rr, x + rr, y + rr], fill=SPORE)

def dust_over(ctx, rng, strength=120):
    """Thick gray powder: translucent wash clipped to the body + speckle."""
    wash = Image.new("RGBA", ctx.img.size, (178, 178, 172, strength))
    alpha = ctx.img.split()[3]
    ctx.img.alpha_composite(Image.composite(wash, Image.new("RGBA", ctx.img.size, (0, 0, 0, 0)), alpha))
    px = ctx.img.load()
    dots = Image.new("RGBA", ctx.img.size, (0, 0, 0, 0))
    dd = ImageDraw.Draw(dots)
    s = ctx.size
    for _ in range(int(s * 2.2)):
        x = rng.uniform(0, 1); y = rng.uniform(0, 1)
        xi, yi = min(s - 1, int(x * s)), min(s - 1, int(y * s))
        if px[xi, yi][3] > 0:
            r = rng.uniform(0.6, 1.6) * s / 64
            dd.ellipse([x * s - r, y * s - r, x * s + r, y * s + r], fill=POWDER_HI)
    ctx.img.alpha_composite(dots)
    ctx.d = ImageDraw.Draw(ctx.img)

def humanoid(size, arch, treat, mid):
    """Front-facing shambling figure. arch tunes the body plan; treat is
    'dusted' | 'stalk' | 'bloom' | 'plain'."""
    rng = rng_for(mid)
    ctx = Ctx(size)
    d = ctx.d
    # body-plan parameters (unit coords)
    P = dict(cx=0.5, head_r=0.082, head_y=0.185, shoulder=0.185, waist=0.115,
             hip_y=0.60, foot_y=0.955, arm_w=0.052, bulky=0.0, short=0.0)
    if arch in ("tough", "brute"):
        P.update(shoulder=0.225, waist=0.145, arm_w=0.068, head_r=0.078)
    if arch == "hulk":
        P.update(shoulder=0.30, waist=0.20, arm_w=0.095, head_r=0.062, head_y=0.16)
    if arch in ("skel_brute",):
        P.update(shoulder=0.235, waist=0.14, arm_w=0.066, head_r=0.080)
    if arch == "skel_hulk":
        P.update(shoulder=0.315, waist=0.205, arm_w=0.092, head_r=0.066, head_y=0.155)
    if arch == "child":
        P.update(head_r=0.098, head_y=0.215, shoulder=0.135, waist=0.088,
                 arm_w=0.040, hip_y=0.56, short=0.10)
    if arch == "fat":
        P.update(bulky=0.085, shoulder=0.205, waist=0.16)
    if arch in ("boomer", "gasbag"):
        P.update(bulky=0.16, shoulder=0.215, waist=0.175, head_r=0.076)
    if arch == "runner":
        P.update(shoulder=0.165, waist=0.10, arm_w=0.044)
    cx = P["cx"]; top = P["short"]
    hy = P["head_y"] + top * 0.4; hr = P["head_r"]
    sh_y = hy + hr + 0.045; hip_y = P["hip_y"] + top * 0.3
    sh, wa = P["shoulder"], P["waist"]
    skin = SKIN; shirt = SHIRT
    if arch == "boomer":
        skin = (138, 126, 116, 255)
    if arch == "gasbag":
        skin = (96, 92, 74, 255); shirt = (52, 50, 44, 255)
    if arch == "rot":
        skin = (106, 112, 88, 255)
    bony = arch in ("skeleton", "skel_brute", "skel_hulk")
    body_col = BONE if bony else skin
    body_hi = BONE_HI if bony else SKIN_HI
    body_dk = BONE_DK if bony else SKIN_DK
    # legs (one shambling step forward)
    for sgn, fwd in ((-1, 0.0), (1, 0.035)):
        lx = cx + sgn * wa * 0.52
        ctx.P([(lx - 0.045, hip_y), (lx + 0.045, hip_y),
               (lx + 0.052 + fwd * sgn, P["foot_y"] - 0.03),
               (lx - 0.030 + fwd * sgn, P["foot_y"] - 0.03)],
              fill=PANTS, outline=OUTLINE)
        ctx.E([lx - 0.052 + fwd * sgn, P["foot_y"] - 0.045, lx + 0.062 + fwd * sgn,
               P["foot_y"] - 0.005], fill=(40, 36, 32, 255), outline=OUTLINE, width=ctx.w(0.01))
    # arms reaching forward-down
    for sgn in (-1, 1):
        sx = cx + sgn * sh; ex = cx + sgn * (sh + 0.055)
        ey = hip_y + 0.10 + (0.02 if sgn > 0 else 0.0)
        ctx.P([(sx - P["arm_w"] / 2, sh_y), (sx + P["arm_w"] / 2, sh_y),
               (ex + P["arm_w"] / 2, ey), (ex - P["arm_w"] / 2, ey)],
              fill=shirt if sgn < 0 else SHIRT_DK, outline=OUTLINE)
        ctx.L([(sx, sh_y + 0.06), (ex, ey)], body_dk, ctx.w(0.030))
        ctx.E([ex - 0.030, ey - 0.012, ex + 0.030, ey + 0.038], fill=body_col,
              outline=OUTLINE, width=ctx.w(0.01))  # hand
    # torso
    ctx.P([(cx - sh, sh_y), (cx + sh, sh_y), (cx + wa, hip_y + 0.03),
           (cx - wa, hip_y + 0.03)], fill=shirt, outline=OUTLINE)
    ctx.P([(cx - sh * 0.82, sh_y + 0.02), (cx - sh * 0.2, sh_y + 0.02),
           (cx - wa * 0.3, hip_y * 0.82), (cx - wa * 0.82, hip_y * 0.82)],
          fill=SHIRT_DK)  # torn shading patch
    if P["bulky"]:
        b = P["bulky"]
        ctx.E([cx - wa - b, sh_y + 0.06, cx + wa + b, hip_y + 0.10],
              fill=skin if arch == "gasbag" else shirt, outline=OUTLINE)
        ctx.E([cx - wa - b * 0.55, sh_y + 0.10, cx + wa * 0.1, hip_y + 0.02],
              fill=SKIN_HI if arch != "gasbag" else (128, 122, 92, 255))
        if arch == "gasbag":  # black/yellow: jaundiced blotches on the swell
            for bx, by, br in ((0.40, 0.46, 0.045), (0.60, 0.52, 0.05), (0.52, 0.40, 0.035)):
                ctx.E([bx - br, by - br, bx + br, by + br], fill=(196, 178, 60, 255))
        if arch == "boomer":  # magenta-tinged blotches
            for bx, by, br in ((0.42, 0.48, 0.04), (0.58, 0.42, 0.033)):
                ctx.E([bx - br, by - br, bx + br, by + br], fill=(168, 92, 130, 255))
    # head
    ctx.E([cx - hr, hy - hr, cx + hr, hy + hr], fill=body_col, outline=OUTLINE)
    ctx.E([cx - hr * 0.62, hy - hr * 0.78, cx + hr * 0.05, hy - hr * 0.05], fill=body_hi)
    if bony:  # skull sockets
        for sgn in (-1, 1):
            ctx.E([cx + sgn * hr * 0.38 - 0.016, hy - 0.012, cx + sgn * hr * 0.38 + 0.016,
                   hy + 0.02], fill=(20, 18, 16, 255))
        ctx.L([(cx - hr * 0.4, hy + hr * 0.62), (cx + hr * 0.4, hy + hr * 0.62)],
              BONE_DK, ctx.w(0.014))
        for i in range(3):
            x = cx - hr * 0.3 + i * hr * 0.3
            ctx.L([(x, hy + hr * 0.5), (x, hy + hr * 0.74)], BONE_DK, ctx.w(0.010))
    elif treat != "bloom" or arch not in ("brute", "hulk"):
        for sgn in (-1, 1):  # sunken eyes
            ctx.E([cx + sgn * hr * 0.36 - 0.011, hy - 0.008, cx + sgn * hr * 0.36 + 0.011,
                   hy + 0.012], fill=(30, 28, 24, 255))
        ctx.L([(cx - hr * 0.3, hy + hr * 0.55), (cx + hr * 0.22, hy + hr * 0.62)],
              SKIN_DK, ctx.w(0.012))  # slack mouth
    # archetype extras
    if bony:  # knobby / twisted bone growths, plated for the hulk
        spots = [(cx - sh - 0.01, sh_y + 0.02), (cx + sh + 0.01, sh_y + 0.02),
                 (cx - sh * 0.6, sh_y + 0.13), (cx + sh * 0.7, sh_y + 0.16),
                 (cx - wa * 0.8, hip_y - 0.05), (cx + wa * 0.85, hip_y - 0.02)]
        for bx, by in spots:
            r = rng.uniform(0.020, 0.036) * (1.5 if arch == "skel_hulk" else 1.0)
            ctx.E([bx - r, by - r, bx + r, by + r], fill=BONE, outline=OUTLINE,
                  width=ctx.w(0.01))
            ctx.E([bx - r * 0.5, by - r * 0.6, bx + r * 0.1, by], fill=BONE_HI)
        if arch == "skel_hulk":  # layered bone plates rising over the torso
            for i in range(3):
                y0 = sh_y + 0.05 + i * 0.11
                ctx.P([(cx - sh * (0.95 - i * 0.1), y0), (cx + sh * (0.95 - i * 0.1), y0),
                       (cx + wa * 0.9, y0 + 0.085), (cx - wa * 0.9, y0 + 0.085)],
                      fill=BONE_DK, outline=OUTLINE)
    if arch == "rot":  # decay: patches, exposed bone, slime-dark stains
        for _ in range(6):
            x = rng.uniform(cx - sh, cx + sh); y = rng.uniform(sh_y, hip_y)
            r = rng.uniform(0.014, 0.03)
            ctx.E([x - r, y - r, x + r, y + r], fill=GORE_DK)
        ctx.E([cx + sh * 0.55, sh_y + 0.10, cx + sh * 0.55 + 0.05, sh_y + 0.16],
              fill=BONE, outline=OUTLINE, width=ctx.w(0.008))
    if arch == "scarred":  # heavy scarring
        for _ in range(5):
            x = rng.uniform(cx - sh * 0.9, cx + sh * 0.9); y = rng.uniform(hy, hip_y)
            ctx.L([(x, y), (x + rng.uniform(0.02, 0.05), y + rng.uniform(0.01, 0.04))],
                  (150, 96, 88, 255), ctx.w(0.012))
    if arch == "acidic":  # prominent yellow veins
        for _ in range(6):
            x = rng.uniform(cx - sh * 0.9, cx + sh * 0.7); y = rng.uniform(sh_y, hip_y - 0.05)
            ctx.L([(x, y), (x + 0.03, y + 0.05), (x + 0.015, y + 0.10)],
                  (206, 190, 60, 255), ctx.w(0.011))
    if arch == "smoker":  # smoke shroud (translucent look -> overlay composite)
        smoke = Image.new("RGBA", ctx.img.size, (0, 0, 0, 0))
        sd = ImageDraw.Draw(smoke)
        s = size
        for _ in range(9):
            x = rng.uniform(0.16, 0.84); y = rng.uniform(0.10, 0.92)
            r = rng.uniform(0.05, 0.11)
            sd.ellipse([(x - r) * s, (y - r * 0.7) * s, (x + r) * s, (y + r * 0.7) * s],
                       fill=(148, 148, 146, 88))
        ctx.img.alpha_composite(smoke)
        ctx.d = ImageDraw.Draw(ctx.img)
    # fungal treatments
    if treat == "stalk":
        stalk(ctx, cx + 0.012, hy - hr * 0.82, h=0.115, lean=0.018,
              cap_r=0.034 if size > 32 else 0.030)
        sprout_cluster(ctx, cx - sh * 0.5, sh_y + 0.05, 0.03, rng, n=4)
    if treat == "bloom":
        stalk(ctx, cx - 0.02, hy - hr * 0.8, h=0.10, lean=-0.02)
        stalk(ctx, cx + sh * 0.55, sh_y + 0.01, h=0.085, lean=0.03, cap_r=0.026)
        stalk(ctx, cx - sh * 0.62, sh_y + 0.03, h=0.07, lean=-0.03, cap_r=0.022)
        stalk(ctx, cx + wa * 0.4, hip_y - 0.06, h=0.06, lean=0.02, cap_r=0.020)
        for bx, by in ((cx - sh * 0.4, sh_y + 0.12), (cx + sh * 0.3, sh_y + 0.2),
                       (cx, hip_y - 0.10), (cx - wa * 0.5, hip_y + 0.02),
                       (cx + hr * 0.4, hy + 0.01)):
            sprout_cluster(ctx, bx, by, 0.035, rng, n=8)
        spore_motes(ctx, rng, n=12)
    if treat == "dusted":
        dust_over(ctx, rng)
    return ctx.img

def crawler(size, treat, mid):
    """Badly damaged legs: torso propped on its arms, legs trailing."""
    rng = rng_for(mid)
    ctx = Ctx(size)
    cx = 0.5
    # trailing ruined legs along the bottom
    for sgn in (-1, 1):
        ctx.P([(cx + sgn * 0.10, 0.66), (cx + sgn * 0.20, 0.66),
               (cx + sgn * 0.30, 0.90), (cx + sgn * 0.17, 0.94)],
              fill=PANTS, outline=OUTLINE)
        ctx.E([cx + sgn * 0.24 - 0.03, 0.885, cx + sgn * 0.24 + 0.045, 0.945],
              fill=GORE_DK, outline=OUTLINE, width=ctx.w(0.01))
    # leaning torso
    ctx.P([(cx - 0.20, 0.30), (cx + 0.20, 0.30), (cx + 0.13, 0.68), (cx - 0.13, 0.68)],
          fill=SHIRT, outline=OUTLINE)
    ctx.P([(cx - 0.16, 0.32), (cx - 0.04, 0.32), (cx - 0.05, 0.62), (cx - 0.11, 0.62)],
          fill=SHIRT_DK)
    # arms planted on the ground
    for sgn in (-1, 1):
        ctx.P([(cx + sgn * 0.17, 0.32), (cx + sgn * 0.235, 0.32),
               (cx + sgn * 0.30, 0.80), (cx + sgn * 0.235, 0.82)],
              fill=SHIRT_DK if sgn > 0 else SHIRT, outline=OUTLINE)
        ctx.E([cx + sgn * 0.265 - 0.032, 0.79, cx + sgn * 0.265 + 0.032, 0.845],
              fill=SKIN, outline=OUTLINE, width=ctx.w(0.01))
    # head low, between the shoulders
    ctx.E([cx - 0.085, 0.115, cx + 0.085, 0.285], fill=SKIN, outline=OUTLINE)
    ctx.E([cx - 0.05, 0.13, cx + 0.01, 0.20], fill=SKIN_HI)
    for sgn in (-1, 1):
        ctx.E([cx + sgn * 0.031 - 0.011, 0.185, cx + sgn * 0.031 + 0.011, 0.205],
              fill=(30, 28, 24, 255))
    if treat == "stalk":
        stalk(ctx, cx + 0.01, 0.115, h=0.10, lean=0.015)
    if treat == "dusted":
        dust_over(ctx, rng)
    return ctx.img

def wretch(size, treat, mid):
    """Prone child; the chest has burst open like a gory flower."""
    rng = rng_for(mid)
    ctx = Ctx(size)
    # body stretched along the ground
    ctx.E([0.10, 0.52, 0.88, 0.80], fill=SHIRT, outline=OUTLINE)
    ctx.E([0.10, 0.52, 0.88, 0.68], fill=SHIRT_DK)
    # legs trailing right
    for dy in (0.0, 0.09):
        ctx.P([(0.80, 0.60 + dy), (0.97, 0.615 + dy), (0.97, 0.665 + dy), (0.80, 0.68 + dy)],
              fill=PANTS, outline=OUTLINE)
    # head at left, serene (fungalize) or slack (dusted)
    ctx.E([0.045, 0.47, 0.235, 0.66], fill=SKIN, outline=OUTLINE)
    ctx.E([0.06, 0.485, 0.13, 0.55], fill=SKIN_HI)
    ctx.L([(0.085, 0.565), (0.125, 0.565)], (30, 28, 24, 255), ctx.w(0.012))  # closed eye
    if treat == "fungalize":
        ctx.L([(0.10, 0.615), (0.15, 0.60)], SKIN_DK, ctx.w(0.010))  # serene smile
    # the burst chest: rib petals around a dark pulsing core
    ccx, ccy = 0.47, 0.635
    for ang in range(8):
        import math
        a = ang * math.pi / 4
        x = ccx + math.cos(a) * 0.105; y = ccy + math.sin(a) * 0.075
        ctx.E([x - 0.030, y - 0.020, x + 0.030, y + 0.020], fill=GORE, outline=OUTLINE,
              width=ctx.w(0.009))
    ctx.E([ccx - 0.052, ccy - 0.038, ccx + 0.052, ccy + 0.038], fill=GORE_DK, outline=OUTLINE)
    ctx.E([ccx - 0.026, ccy - 0.018, ccx + 0.026, ccy + 0.018], fill=(168, 66, 58, 255))
    ctx.E([ccx - 0.012, ccy - 0.009, ccx + 0.008, ccy + 0.006], fill=(206, 120, 104, 255))
    if treat == "stalk":
        stalk(ctx, 0.135, 0.475, h=0.115, lean=-0.01)
        sprout_cluster(ctx, 0.62, 0.56, 0.03, rng, n=5)
    if treat == "dusted":
        dust_over(ctx, rng)
    return ctx.img

def raptor(size, treat, mid):
    """Small winged spawn: three spongy hand-like wings, jagged spike."""
    rng = rng_for(mid)
    ctx = Ctx(size)
    import math
    cx, cy = 0.5, 0.52
    # three wings fanning up/out: left, right, and one rearing behind
    for ang_deg, ln in ((-155, 0.34), (-25, 0.34), (-90, 0.30)):
        a = math.radians(ang_deg)
        tx, ty = cx + math.cos(a) * ln, cy + math.sin(a) * ln * 0.8
        # wing membrane: elongated palm
        perp = a + math.pi / 2
        wdt = 0.055
        pts = [(cx + math.cos(perp) * wdt, cy + math.sin(perp) * wdt * 0.8),
               (tx + math.cos(perp) * wdt * 0.8, ty + math.sin(perp) * wdt * 0.6),
               (tx, ty)]
        pts.append((tx - math.cos(perp) * wdt * 0.8, ty - math.sin(perp) * wdt * 0.6))
        pts.append((cx - math.cos(perp) * wdt, cy - math.sin(perp) * wdt * 0.8))
        ctx.P(pts, fill=(150, 148, 128, 255), outline=OUTLINE)
        # fingers: thin rays from palm to tip
        for f in (-0.6, 0.0, 0.6):
            fa = a + f * 0.35
            fx, fy = cx + math.cos(fa) * ln * 1.04, cy + math.sin(fa) * ln * 0.84
            ctx.L([(cx, cy), (fx, fy)], (108, 106, 90, 255), ctx.w(0.014))
    # jagged spike at the wing junction
    ctx.P([(cx - 0.035, cy - 0.06), (cx - 0.008, cy - 0.20), (cx + 0.012, cy - 0.115),
           (cx + 0.026, cy - 0.185), (cx + 0.042, cy - 0.06)], fill=BONE, outline=OUTLINE)
    # small body
    ctx.E([cx - 0.075, cy - 0.06, cx + 0.075, cy + 0.11], fill=SKIN_DK, outline=OUTLINE)
    ctx.E([cx - 0.045, cy - 0.045, cx + 0.01, cy + 0.02], fill=SKIN)
    ctx.E([cx - 0.016, cy - 0.01, cx + 0.016, cy + 0.022], fill=GORE_DK)  # maw
    if treat == "stalk":
        stalk(ctx, cx + 0.055, cy - 0.03, h=0.085, lean=0.025, cap_r=0.024)
    if treat == "dusted":
        dust_over(ctx, rng)
    return ctx.img

def mound(size, construct, scale, mid):
    """Pale pile of fungus growing over a mass of gore; constructs get
    interwoven root limbs that drag the pulp around."""
    rng = rng_for(mid)
    ctx = Ctx(size)
    cx = 0.5
    base_y = 0.90
    hump = 0.52 if scale == 1 else (0.58 if scale == 2 else 0.66)
    # gore mass spilling at the base
    for i in range(7):
        x = rng.uniform(0.14, 0.86); r = rng.uniform(0.05, 0.10)
        ctx.E([x - r, base_y - r * 0.5, x + r, base_y + r * 0.42],
              fill=GORE_DK if i % 2 else GORE, outline=OUTLINE, width=ctx.w(0.009))
    if construct:  # root/vine limbs
        for sgn in (-1, 1):
            pts = [(cx + sgn * 0.22, 0.62), (cx + sgn * 0.40, 0.70),
                   (cx + sgn * 0.46, 0.84), (cx + sgn * 0.40, 0.93)]
            ctx.L(pts, OUTLINE, ctx.w(0.045))
            ctx.L(pts, (96, 102, 78, 255), ctx.w(0.026))
            pts2 = [(cx + sgn * 0.16, 0.72), (cx + sgn * 0.33, 0.80), (cx + sgn * 0.30, 0.94)]
            ctx.L(pts2, OUTLINE, ctx.w(0.034))
            ctx.L(pts2, (110, 116, 88, 255), ctx.w(0.018))
    # the pale pile: overlapping humps
    humps = [(0.50, 0.60, 0.30, hump), (0.33, 0.70, 0.22, 0.30), (0.67, 0.70, 0.22, 0.30),
             (0.44, 0.52, 0.18, 0.26), (0.60, 0.55, 0.16, 0.22)]
    for hx, hy, hw, hh in humps:
        ctx.E([hx - hw, hy - hh * 0.55, hx + hw, hy + hh * 0.55], fill=(206, 204, 194, 255),
              outline=OUTLINE)
    for hx, hy, hw, hh in humps[:3]:
        ctx.E([hx - hw * 0.6, hy - hh * 0.42, hx + hw * 0.05, hy + hh * 0.05],
              fill=(232, 230, 220, 255))
    for hx, hy, hw, hh in humps[1:]:
        ctx.E([hx + hw * 0.1, hy + hh * 0.1, hx + hw * 0.75, hy + hh * 0.48],
              fill=(178, 176, 166, 255))
    # sprouts and stalks all over the pile
    stalk(ctx, 0.40, 0.36, h=0.075, lean=-0.015, cap_r=0.024)
    stalk(ctx, 0.62, 0.42, h=0.06, lean=0.02, cap_r=0.020)
    for _ in range(6):
        x = rng.uniform(0.24, 0.76); y = rng.uniform(0.42, 0.74)
        sprout_cluster(ctx, x, y, 0.028, rng, n=6)
    if scale == 3:  # massive: unnaturally hunched bulk, heavier sprouting
        ctx.E([0.30, 0.16, 0.70, 0.44], fill=(196, 194, 184, 255), outline=OUTLINE)
        ctx.E([0.36, 0.19, 0.56, 0.32], fill=(224, 222, 212, 255))
        stalk(ctx, 0.50, 0.18, h=0.07, lean=0.0, cap_r=0.026)
        spore_motes(ctx, rng, n=10)
    return ctx.img

def quadruped(size, tiger, mid):
    """Zougar (fungal cat) / ziger (tiger wight): side view, fungal pawpads."""
    rng = rng_for(mid)
    ctx = Ctx(size)
    body = (172, 170, 160, 255) if not tiger else (156, 114, 72, 255)
    body_hi = (198, 196, 186, 255) if not tiger else (182, 140, 92, 255)
    body_dk = (128, 126, 116, 255) if not tiger else (116, 82, 50, 255)
    # tail
    ctx.L([(0.16, 0.42), (0.07, 0.30), (0.10, 0.16)], OUTLINE, ctx.w(0.040))
    ctx.L([(0.16, 0.42), (0.07, 0.30), (0.10, 0.16)], body_dk, ctx.w(0.022))
    # legs
    for lx in (0.26, 0.38, 0.64, 0.76):
        ctx.P([(lx - 0.032, 0.55), (lx + 0.032, 0.55), (lx + 0.028, 0.86), (lx - 0.028, 0.86)],
              fill=body_dk if lx in (0.38, 0.64) else body, outline=OUTLINE)
        # fungal bulbs extending the pawpads
        ctx.E([lx - 0.052, 0.825, lx + 0.052, 0.905], fill=(206, 204, 192, 255), outline=OUTLINE,
              width=ctx.w(0.009))
        if not tiger:  # zougar spews spores from its pawpads
            spore_motes(ctx, rng, n=4, region=(lx - 0.07, 0.84, lx + 0.07, 0.97))
    # body
    ctx.E([0.14, 0.30, 0.82, 0.62], fill=body, outline=OUTLINE)
    ctx.E([0.20, 0.33, 0.60, 0.50], fill=body_hi)
    if tiger:  # rotting tiger stripes
        for i in range(6):
            x = 0.24 + i * 0.085
            ctx.P([(x, 0.315), (x + 0.028, 0.315), (x + 0.012, 0.50), (x - 0.006, 0.50)],
                  fill=(52, 40, 30, 255))
    # head, stiffly forward (ziger's "head crackles when trying to turn")
    ctx.E([0.76, 0.26, 0.99, 0.52], fill=body, outline=OUTLINE)
    ctx.E([0.80, 0.29, 0.92, 0.40], fill=body_hi)
    ctx.P([(0.80, 0.27), (0.83, 0.17), (0.87, 0.26)], fill=body_dk, outline=OUTLINE)  # ear
    ctx.E([0.885, 0.345, 0.915, 0.375], fill=(30, 28, 24, 255))  # eye
    ctx.P([(0.93, 0.44), (0.99, 0.46), (0.93, 0.50)], fill=body_dk, outline=OUTLINE)  # muzzle
    if tiger:
        for i in range(2):
            x = 0.80 + i * 0.07
            ctx.P([(x, 0.265), (x + 0.02, 0.265), (x + 0.01, 0.36), (x - 0.004, 0.36)],
                  fill=(52, 40, 30, 255))
    # fungal growth on the back
    stalk(ctx, 0.45, 0.315, h=0.075, lean=-0.01, cap_r=0.024)
    sprout_cluster(ctx, 0.62, 0.33, 0.03, rng, n=6)
    return ctx.img

def H(arch, treat):
    return lambda size, mid: humanoid(size, arch, treat, mid)

JOBS = [
    # --- 32x32: standard humanoids and small creatures ---
    ("mon_zombie_dusted", 32, H("zombie", "dusted")),
    ("mon_zombie_fungalize", 32, H("zombie", "stalk")),
    ("mon_zombie_smoker_dusted", 32, H("smoker", "dusted")),
    ("mon_zombie_smoker_fungalize", 32, H("smoker", "stalk")),
    ("mon_skeleton_dusted", 32, H("skeleton", "dusted")),
    ("mon_skeleton_fungalize", 32, H("skeleton", "stalk")),
    ("mon_skeleton_fungus", 32, H("skeleton", "bloom")),
    ("mon_zombie_child_dusted", 32, H("child", "dusted")),
    ("mon_zombie_child_fungalize", 32, H("child", "stalk")),
    ("mon_zombie_wretch_dusted", 32, lambda s, m: wretch(s, "dusted", m)),
    ("mon_zombie_wretch_fungalize", 32, lambda s, m: wretch(s, "stalk", m)),
    ("mon_spawn_raptor_dusted", 32, lambda s, m: raptor(s, "dusted", m)),
    ("mon_spawn_raptor_fungalize", 32, lambda s, m: raptor(s, "stalk", m)),
    ("mon_zombie_runner_dusted", 32, H("runner", "dusted")),
    ("mon_zombie_runner_fungalize", 32, H("runner", "stalk")),
    ("mon_zombie_rot_dusted", 32, H("rot", "dusted")),
    ("mon_zombie_rot_fungalize", 32, H("rot", "stalk")),
    ("mon_zombie_tough_dusted", 32, H("tough", "dusted")),
    ("mon_zombie_tough_fungalize", 32, H("tough", "stalk")),
    ("mon_zombie_crawler_dusted", 32, lambda s, m: crawler(s, "dusted", m)),
    ("mon_zombie_crawler_fungalize", 32, lambda s, m: crawler(s, "stalk", m)),
    ("mon_zombie_fat_dusted", 32, H("fat", "dusted")),
    ("mon_zombie_fat_fungalize", 32, H("fat", "stalk")),
    ("mon_beekeeper_dusted", 32, H("scarred", "dusted")),
    ("mon_beekeeper_fungalize", 32, H("scarred", "stalk")),
    ("mon_zombie_acidic_dusted", 32, H("acidic", "dusted")),
    ("mon_zombie_acidic_fungalize", 32, H("acidic", "stalk")),
    ("mon_fungal_mound", 32, lambda s, m: mound(s, False, 1, m)),
    ("mon_fungal_construct", 32, lambda s, m: mound(s, True, 1, m)),
    ("mon_zougar_fungus", 32, lambda s, m: quadruped(s, False, m)),
    # --- 64x64: bulky bodies ---
    ("mon_gasbag_dusted", 64, H("gasbag", "dusted")),
    ("mon_gasbag_fungalize", 64, H("gasbag", "stalk")),
    ("mon_boomer_dusted", 64, H("boomer", "dusted")),
    ("mon_boomer_fungalize", 64, H("boomer", "stalk")),
    ("mon_skeleton_brute_dusted", 64, H("skel_brute", "dusted")),
    ("mon_skeleton_brute_fungalize", 64, H("skel_brute", "stalk")),
    ("mon_skeleton_brute_fungus", 64, H("skel_brute", "bloom")),
    ("mon_zombie_brute_dusted", 64, H("brute", "dusted")),
    ("mon_zombie_brute_fungalize", 64, H("brute", "stalk")),
    ("mon_zombie_brute_fungal", 64, H("brute", "bloom")),
    ("mon_fungal_mound_large", 64, lambda s, m: mound(s, False, 2, m)),
    ("mon_fungal_construct_large", 64, lambda s, m: mound(s, True, 2, m)),
    ("mon_ziger_fungus", 64, lambda s, m: quadruped(s, True, m)),
    # --- 96x96: hulks and massive mounds ---
    ("mon_skeleton_hulk_dusted", 96, H("skel_hulk", "dusted")),
    ("mon_skeleton_hulk_fungalize", 96, H("skel_hulk", "stalk")),
    ("mon_zombie_hulk_dusted", 96, H("hulk", "dusted")),
    ("mon_zombie_hulk_fungalize", 96, H("hulk", "stalk")),
    ("mon_zombie_hulk_fungal", 96, H("hulk", "bloom")),
    ("mon_fungal_mound_massive", 96, lambda s, m: mound(s, False, 3, m)),
    ("mon_fungal_construct_massive", 96, lambda s, m: mound(s, True, 3, m)),
]

SHEET = {32: "pngs_normal_32x32", 64: "pngs_large_64x64", 96: "pngs_giant_96x96"}
CREDITS = ("Drawn from the monster's in-game description for the CDDA Voxel\n"
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.\n")

for mid, size, draw in JOBS:
    folder = os.path.join(BASE, SHEET[size], "monsters", mid)
    os.makedirs(folder, exist_ok=True)
    img = draw(size, mid)
    img.save(os.path.join(folder, mid + ".png"))
    with open(os.path.join(folder, mid + ".json"), "w") as f:
        json.dump([{"id": mid, "fg": mid, "bg": ""}], f, indent=2)
        f.write("\n")
    with open(os.path.join(folder, "credits.txt"), "w") as f:
        f.write(CREDITS)
    print("wrote", mid, img.size)

# contact sheet: every sprite over a neutral backdrop, 10 per row
cols = 10
rows = (len(JOBS) + cols - 1) // cols
cell = 64
sheet = Image.new("RGBA", (cols * cell, rows * cell), (58, 62, 54, 255))
for i, (mid, size, _draw) in enumerate(JOBS):
    p = os.path.join(BASE, SHEET[size], "monsters", mid, mid + ".png")
    im = Image.open(p)
    if im.width != cell:
        im = im.resize((cell, cell), Image.NEAREST)
    sheet.alpha_composite(im, ((i % cols) * cell, (i // cols) * cell))
sheet.save(os.path.join(BASE, "preview_fungus_zombies.png"))
print("wrote preview_fungus_zombies.png", sheet.size, "jobs:", len(JOBS))
