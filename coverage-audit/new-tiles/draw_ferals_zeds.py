#!/usr/bin/env python3
"""
Draw the missing feral-human / zed-cluster sprites from their descriptions.

Sources (fork-src/cdda/data/json/monsters/, CDDA 6a3e59e0):
  feral_humans.json (14) -- living humans gone feral: sailors with wrench /
    crash axe / mop / lug wrench, a Marine with rifle+bayonet, an officer
    with a service pistol, a fungus-infected cop in overgrown body armor,
    a shiv-wielding prisoner, and six serum test subjects in orange
    jumpsuit remnants (glowing-healthy, pipe-swinging, unscratched,
    sped-up/jerky, spear+ cobbled armor, darkness-blending). All carry
    looks_like "chud" (a dead-end string, not a monster id) so they need
    real sprites. Shared feral tells: bulging blackened veins, bloodshot
    wide eyes, twitching hunched posture.
  zed_misc.json (7) -- upright summoner (black_red), two regenerating
    quadrupeds (exposed churning muscle; the final is a dismantled-and-
    reassembled skeleton shape), an ocular zombie whose head is filled
    with eyes, a fursuited zombie caked in black goo, a crawling decayed
    grasper, and a glistening plaguebearer.
  zed_amphibian.json (12) -- zombie frogs/toads: colossal bullfrog,
    bioelectric frog (arcs across skin), bloated toad (sludge), treefrog
    (pale swivelling eyes), shadow leaper (flat, wreathed in darkness),
    warted monstrosity (sludge-bursting warts), death-mask toad (ossified
    face plate + beak), leaping husk (desiccated, cracked), batrachian
    mummy and smoker hulk (papery / smoke-fuming), gastro bufo (acid
    leaking from skin).
  zed_amalgamation.json (7) -- the amalgamations are NOT single fused
    humanoids in the data; each is its own creature plan, drawn as
    described: breather (featureless pink breathing flesh sac) and its
    hub, spotter (inflated white body on five dainty legs, two limbs
    tipped with head-sized compound eyes), corroder (gangly six-legger,
    green pustules feeding veins to blackened claws, jagged proboscis
    dripping caustic), jumper (feline-gait quadruped holding an almost-
    translucent membrane, raven beak with jagged black teeth), zapper
    (crablike, white armor plates, glowing organ, spark-tipped raised
    appendage), soldier (bear-like armored bulk, clubbed tail, talon
    limbs framing a vertical mouth on a smooth featureless head).

Design: one parameterized front-facing humanoid renderer (living-skin
and zombie-skin palettes, uniform/clothing colors, weapon props, feral
vein/eye tells), a crawler and two quadruped variants, a fursuit figure,
a parameterized front-facing frog/toad renderer, and dedicated
amalgamation renderers. Sheet placement by resolved volume (copy-from
chain): <100 L -> 32, 100-400 L -> 64, >400 L -> 96. Translucent
effects (speed-blur ghosts, shadow wreath, smoke haze, jumper membrane)
are painted on overlays and composited with Image.alpha_composite --
never semi-transparent body pixels. Dark outline, flat fills + one
highlight pass -- UltiCa monster style. Output is UltiCa decomposed
layout (png + tile-entry json per folder). This script only ever
creates/overwrites its own 40 folders.
"""
from PIL import Image, ImageDraw
import json, math, os, random, zlib

BASE = os.path.dirname(os.path.abspath(__file__))

def rng_for(name):
    return random.Random(zlib.crc32(name.encode()))

OUTLINE = (24, 26, 22, 255)
# living human skin (ferals are alive, not corpses)
HSKIN = (198, 152, 116, 255); HSKIN_HI = (228, 184, 144, 255); HSKIN_DK = (164, 118, 84, 255)
HEALTHY = (214, 168, 128, 255); HEALTHY_HI = (240, 202, 160, 255)  # serum glow
# zombie skin
ZSKIN = (118, 132, 98, 255); ZSKIN_HI = (148, 162, 124, 255); ZSKIN_DK = (88, 102, 72, 255)
GORE = (122, 42, 37, 255); GORE_DK = (82, 28, 26, 255)
BONE = (216, 208, 188, 255); BONE_HI = (236, 230, 212, 255); BONE_DK = (166, 156, 136, 255)
VEIN = (38, 34, 40, 255)          # blackened bulging veins
STEEL = (150, 154, 160, 255); STEEL_DK = (96, 100, 108, 255)
NAVY = (40, 54, 100, 255); NAVY_DK = (27, 38, 72, 255)
ORANGE = (216, 110, 32, 255); ORANGE_DK = (178, 84, 22, 255)

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
    """Short gray fungal stalk with cap, rooted at (x, y), growing up."""
    top_x, top_y = x + lean, y - h
    ctx.L([(x, y), ((x + top_x) / 2 + lean * 0.3, (y + top_y) / 2), (top_x, top_y)],
          OUTLINE, ctx.w(0.030))
    ctx.L([(x, y), ((x + top_x) / 2 + lean * 0.3, (y + top_y) / 2), (top_x, top_y)],
          (188, 188, 178, 255), ctx.w(0.016))
    ctx.E([top_x - cap_r, top_y - cap_r * 0.75, top_x + cap_r, top_y + cap_r * 0.75],
          fill=(148, 146, 136, 255), outline=OUTLINE, width=ctx.w(0.012))
    ctx.E([top_x - cap_r * 0.5, top_y - cap_r * 0.55, top_x + cap_r * 0.1, top_y],
          fill=(208, 206, 194, 255))

def sprout_cluster(ctx, cx, cy, r, rng, n=7):
    for _ in range(n):
        x = cx + rng.uniform(-r, r); y = cy + rng.uniform(-r * 0.7, r * 0.7)
        rr = rng.uniform(0.006, 0.016)
        ctx.E([x - rr, y - rr, x + rr, y + rr], fill=(172, 174, 160, 255),
              outline=OUTLINE, width=ctx.w(0.008))

def overlay_ellipse(ctx, box, fill):
    """Paint one translucent ellipse via a composited overlay."""
    ov = Image.new("RGBA", ctx.img.size, (0, 0, 0, 0))
    od = ImageDraw.Draw(ov)
    s = ctx.size
    od.ellipse([box[0] * s, box[1] * s, box[2] * s, box[3] * s], fill=fill)
    ctx.img.alpha_composite(ov)
    ctx.d = ImageDraw.Draw(ctx.img)

# ---------------------------------------------------------------- weapons
def draw_weapon(ctx, kind, hx, hy):
    """Prop in the right hand at (hx, hy), unit coords."""
    if kind == "wrench":  # massive wrench, jaw up
        ctx.L([(hx, hy + 0.10), (hx + 0.015, hy - 0.06)], OUTLINE, ctx.w(0.052))
        ctx.L([(hx, hy + 0.10), (hx + 0.015, hy - 0.06)], STEEL, ctx.w(0.034))
        ctx.E([hx - 0.005, hy - 0.115, hx + 0.055, hy - 0.045], fill=STEEL, outline=OUTLINE,
              width=ctx.w(0.01))
        ctx.E([hx + 0.018, hy - 0.125, hx + 0.052, hy - 0.088], fill=(0, 0, 0, 0))
        ctx.P([(hx + 0.030, hy - 0.118), (hx + 0.058, hy - 0.100), (hx + 0.030, hy - 0.078)],
              fill=STEEL)
    elif kind == "lug":  # four-way lug wrench
        ctx.L([(hx - 0.045, hy - 0.01), (hx + 0.055, hy - 0.01)], OUTLINE, ctx.w(0.040))
        ctx.L([(hx - 0.045, hy - 0.01), (hx + 0.055, hy - 0.01)], STEEL_DK, ctx.w(0.024))
        ctx.L([(hx + 0.005, hy - 0.075), (hx + 0.005, hy + 0.055)], OUTLINE, ctx.w(0.040))
        ctx.L([(hx + 0.005, hy - 0.075), (hx + 0.005, hy + 0.055)], STEEL_DK, ctx.w(0.024))
        for ex, ey in ((-0.045, -0.01), (0.055, -0.01), (0.005, -0.075), (0.005, 0.055)):
            ctx.E([hx + ex - 0.016, hy + ey - 0.016, hx + ex + 0.016, hy + ey + 0.016],
                  fill=STEEL, outline=OUTLINE, width=ctx.w(0.008))
    elif kind == "axe":  # crash axe, red head
        ctx.L([(hx - 0.03, hy + 0.13), (hx + 0.035, hy - 0.09)], OUTLINE, ctx.w(0.036))
        ctx.L([(hx - 0.03, hy + 0.13), (hx + 0.035, hy - 0.09)], (120, 84, 52, 255), ctx.w(0.020))
        ctx.P([(hx + 0.005, hy - 0.115), (hx + 0.085, hy - 0.095), (hx + 0.088, hy - 0.045),
               (hx + 0.02, hy - 0.055)], fill=(178, 44, 36, 255), outline=OUTLINE)
        ctx.P([(hx + 0.005, hy - 0.115), (hx - 0.028, hy - 0.085), (hx + 0.012, hy - 0.062)],
              fill=STEEL_DK, outline=OUTLINE)
    elif kind == "mop":  # stout mop, head down at the floor
        ctx.L([(hx - 0.075, hy - 0.16), (hx + 0.045, hy + 0.10)], OUTLINE, ctx.w(0.034))
        ctx.L([(hx - 0.075, hy - 0.16), (hx + 0.045, hy + 0.10)], (128, 96, 60, 255), ctx.w(0.018))
        for i in range(5):
            ox = (i - 2) * 0.016
            ctx.L([(hx + 0.045 + ox * 0.4, hy + 0.09), (hx + 0.055 + ox, hy + 0.185)],
                  (196, 190, 168, 255), ctx.w(0.013))
    elif kind == "rifle":  # rifle with gleaming bayonet, diagonal two-hand
        ctx.L([(0.30, 0.78), (0.76, 0.34)], OUTLINE, ctx.w(0.046))
        ctx.L([(0.30, 0.78), (0.76, 0.34)], (66, 52, 38, 255), ctx.w(0.028))
        ctx.L([(0.72, 0.375), (0.80, 0.295)], (30, 30, 32, 255), ctx.w(0.020))  # barrel
        ctx.P([(0.80, 0.295), (0.885, 0.225), (0.845, 0.285)], fill=(226, 230, 234, 255),
              outline=OUTLINE)  # bayonet
        ctx.R([0.44, 0.545, 0.52, 0.615], fill=(40, 40, 44, 255), outline=OUTLINE)  # magazine
    elif kind == "pistol":
        ctx.P([(hx, hy - 0.01), (hx + 0.075, hy - 0.01), (hx + 0.075, hy + 0.018),
               (hx + 0.022, hy + 0.018), (hx + 0.016, hy + 0.055), (hx, hy + 0.05)],
              fill=(36, 36, 40, 255), outline=OUTLINE)
    elif kind == "baton":
        ctx.L([(hx - 0.01, hy + 0.09), (hx + 0.045, hy - 0.075)], OUTLINE, ctx.w(0.042))
        ctx.L([(hx - 0.01, hy + 0.09), (hx + 0.045, hy - 0.075)], (44, 44, 50, 255), ctx.w(0.026))
        ctx.L([(hx + 0.005, hy + 0.028), (hx + 0.045, hy + 0.045)], (44, 44, 50, 255), ctx.w(0.016))
    elif kind == "shiv":
        ctx.L([(hx, hy + 0.03), (hx + 0.02, hy + 0.065)], (90, 70, 50, 255), ctx.w(0.018))
        ctx.P([(hx + 0.012, hy - 0.055), (hx + 0.036, hy + 0.032), (hx + 0.002, hy + 0.032)],
              fill=(214, 218, 222, 255), outline=OUTLINE)
    elif kind == "pipe":
        ctx.L([(hx - 0.02, hy + 0.12), (hx + 0.03, hy - 0.10)], OUTLINE, ctx.w(0.044))
        ctx.L([(hx - 0.02, hy + 0.12), (hx + 0.03, hy - 0.10)], STEEL_DK, ctx.w(0.027))
        ctx.E([hx + 0.012, hy - 0.128, hx + 0.052, hy - 0.088], fill=STEEL, outline=OUTLINE,
              width=ctx.w(0.008))  # fitting
    elif kind == "spear":  # homemade spear, long haft + pale blade
        ctx.L([(hx - 0.05, hy + 0.24), (hx + 0.045, hy - 0.20)], OUTLINE, ctx.w(0.032))
        ctx.L([(hx - 0.05, hy + 0.24), (hx + 0.045, hy - 0.20)], (122, 92, 56, 255), ctx.w(0.017))
        ctx.P([(hx + 0.045, hy - 0.20), (hx + 0.075, hy - 0.145), (hx + 0.028, hy - 0.115)],
              fill=(206, 210, 214, 255), outline=OUTLINE)

# ------------------------------------------------------------- humanoids
def humanoid(size, cfg, mid):
    """Front-facing figure. cfg keys: skin/hi/dk, shirt/shirt_dk, pants,
    weapon, hunch (0 upright .. 1 deep), plus boolean tells: veins,
    bloodshot, healthy, stains (rust/blood), oil, hair_wild, vest,
    fungal, cobbled (armor patches), blur (speed ghosts), dark (shadow
    blend), goo (black caking), sheen (wet glisten), mouth_open."""
    rng = rng_for(mid)
    ctx = Ctx(size)
    skin = cfg.get("skin", ZSKIN); skin_hi = cfg.get("skin_hi", ZSKIN_HI)
    skin_dk = cfg.get("skin_dk", ZSKIN_DK)
    shirt = cfg.get("shirt", (88, 79, 66, 255)); shirt_dk = cfg.get("shirt_dk", (64, 57, 47, 255))
    pants = cfg.get("pants", (56, 57, 66, 255))
    hunch = cfg.get("hunch", 0.55)
    cx = 0.5
    hr = 0.082
    hy = 0.175 + hunch * 0.055
    sh_y = hy + hr + 0.045
    hip_y = 0.615
    sh, wa = 0.180, 0.112
    foot_y = 0.955
    # speed-blur ghosts first (landru): offset echoes of the whole figure
    ghost_boxes = None
    if cfg.get("blur"):
        ghost_boxes = []
        for gx in (-0.055, 0.055):
            ov = Image.new("RGBA", (size, size), (0, 0, 0, 0))
            od = ImageDraw.Draw(ov)
            s = size
            od.ellipse([(cx + gx - 0.115) * s, (hip_y - 0.02) * s, (cx + gx + 0.115) * s, (foot_y - 0.30) * s],
                       fill=(*ORANGE[:3], 52))
            od.ellipse([(cx + gx - hr) * s, (hy - hr) * s, (cx + gx + hr) * s, (hy + hr) * s],
                       fill=(*HSKIN[:3], 52))
            ctx.img.alpha_composite(ov)
        ctx.d = ImageDraw.Draw(ctx.img)
    # legs
    for sgn, fwd in ((-1, 0.0), (1, 0.030)):
        lx = cx + sgn * wa * 0.52
        ctx.P([(lx - 0.045, hip_y), (lx + 0.045, hip_y),
               (lx + 0.052 + fwd * sgn, foot_y - 0.03), (lx - 0.030 + fwd * sgn, foot_y - 0.03)],
              fill=pants, outline=OUTLINE)
        ctx.E([lx - 0.052 + fwd * sgn, foot_y - 0.045, lx + 0.062 + fwd * sgn, foot_y - 0.005],
              fill=(40, 36, 32, 255), outline=OUTLINE, width=ctx.w(0.01))
    # torso
    ctx.P([(cx - sh, sh_y), (cx + sh, sh_y), (cx + wa, hip_y + 0.03), (cx - wa, hip_y + 0.03)],
          fill=shirt, outline=OUTLINE)
    ctx.P([(cx - sh * 0.82, sh_y + 0.02), (cx - sh * 0.2, sh_y + 0.02),
           (cx - wa * 0.3, hip_y * 0.86), (cx - wa * 0.82, hip_y * 0.86)], fill=shirt_dk)
    if cfg.get("oil"):  # grease/oil blotches
        for _ in range(6):
            x = rng.uniform(cx - sh * 0.9, cx + sh * 0.9); y = rng.uniform(sh_y + 0.02, hip_y)
            r = rng.uniform(0.014, 0.032)
            ctx.E([x - r, y - r * 0.7, x + r, y + r * 0.7], fill=(30, 28, 26, 255))
    if cfg.get("stains"):  # dried viscera / blood swaths
        for _ in range(7):
            x = rng.uniform(cx - sh, cx + sh); y = rng.uniform(sh_y, hip_y + 0.02)
            r = rng.uniform(0.016, 0.038)
            ctx.E([x - r, y - r * 0.75, x + r, y + r * 0.75],
                  fill=(112, 52, 38, 255) if rng.random() < 0.6 else (94, 44, 34, 255))
    if cfg.get("vest"):  # cop body armor over the shirt
        ctx.P([(cx - sh * 0.78, sh_y + 0.03), (cx + sh * 0.78, sh_y + 0.03),
               (cx + wa * 0.82, hip_y - 0.01), (cx - wa * 0.82, hip_y - 0.01)],
              fill=(38, 40, 46, 255), outline=OUTLINE)
        ctx.L([(cx - sh * 0.5, sh_y + 0.10), (cx + sh * 0.5, sh_y + 0.10)],
              (58, 60, 68, 255), ctx.w(0.014))
    if cfg.get("cobbled"):  # riot-armor/lab-supply patches over the jumpsuit
        for bx, by, bw, bh, col in ((-0.115, 0.05, 0.105, 0.13, (52, 56, 64, 255)),
                                   (0.012, 0.045, 0.10, 0.15, (168, 172, 176, 255)),
                                   (-0.055, 0.21, 0.115, 0.10, (44, 48, 56, 255))):
            ctx.R([cx + bx, sh_y + by, cx + bx + bw, sh_y + by + bh], fill=col, outline=OUTLINE)
        ctx.L([(cx - 0.13, sh_y + 0.30), (cx - 0.10, hip_y - 0.02)], (52, 56, 64, 255),
              ctx.w(0.030))  # shin guard hint
    # arms (weapon arm reaches lower)
    weapon = cfg.get("weapon")
    for sgn in (-1, 1):
        sx = cx + sgn * sh
        ex = cx + sgn * (sh + 0.055)
        ey = hip_y + 0.10 + (0.02 if sgn > 0 else 0.0)
        if weapon in ("rifle", "mop", "spear") and sgn < 0:
            ex, ey = cx - 0.10, 0.60  # off hand up the haft
        ctx.P([(sx - 0.026, sh_y), (sx + 0.026, sh_y), (ex + 0.026, ey), (ex - 0.026, ey)],
              fill=shirt if sgn < 0 else shirt_dk, outline=OUTLINE)
        ctx.E([ex - 0.030, ey - 0.012, ex + 0.030, ey + 0.038], fill=skin,
              outline=OUTLINE, width=ctx.w(0.01))
        if cfg.get("veins"):  # bulging veins down the forearm
            ctx.L([(sx + sgn * 0.01, sh_y + 0.07), (ex - sgn * 0.008, ey - 0.03)],
                  VEIN, ctx.w(0.011))
    # weapon behind the hand line for two-handers, else in right hand
    hx, hy2 = cx + sh + 0.055, hip_y + 0.10 + 0.02
    if weapon:
        draw_weapon(ctx, weapon, hx, hy2)
        # redraw the gripping hand over the haft
        ctx.E([hx - 0.028, hy2 - 0.010, hx + 0.028, hy2 + 0.034], fill=skin,
              outline=OUTLINE, width=ctx.w(0.01))
    # head
    ctx.E([cx - hr, hy - hr, cx + hr, hy + hr], fill=skin, outline=OUTLINE)
    ctx.E([cx - hr * 0.62, hy - hr * 0.78, cx + hr * 0.05, hy - hr * 0.05], fill=skin_hi)
    if cfg.get("hair_wild"):  # tangled hair spikes
        for i in range(6):
            a = math.pi * (0.08 + 0.84 * i / 5)
            x0 = cx + math.cos(a) * hr * 0.85; y0 = hy - math.sin(a) * hr * 0.8
            x1 = cx + math.cos(a) * hr * 1.35; y1 = hy - math.sin(a) * hr * 1.25
            ctx.L([(x0, y0), (x1, y1)], (52, 42, 32, 255), ctx.w(0.016))
    else:  # hair cap
        ctx.P([(cx - hr, hy - hr * 0.35), (cx - hr * 0.9, hy - hr * 0.95),
               (cx, hy - hr * 1.02), (cx + hr * 0.9, hy - hr * 0.95), (cx + hr, hy - hr * 0.35),
               (cx + hr * 0.55, hy - hr * 0.55), (cx - hr * 0.55, hy - hr * 0.55)],
              fill=cfg.get("hair", (58, 48, 38, 255)))
    # eyes: wide and staring; bloodshot = pale sclera + red flecks
    for sgn in (-1, 1):
        exx = cx + sgn * hr * 0.37
        if cfg.get("bloodshot") or cfg.get("living"):
            ctx.E([exx - 0.016, hy - 0.014, exx + 0.016, hy + 0.014], fill=(232, 226, 214, 255),
                  outline=OUTLINE, width=ctx.w(0.007))
            ctx.E([exx - 0.006, hy - 0.005, exx + 0.006, hy + 0.007], fill=(26, 24, 22, 255))
            if cfg.get("bloodshot"):
                ctx.L([(exx - 0.014, hy + 0.008), (exx - 0.004, hy + 0.003)], (178, 52, 44, 255),
                      ctx.w(0.008))
                ctx.L([(exx + 0.013, hy - 0.010), (exx + 0.004, hy - 0.002)], (178, 52, 44, 255),
                      ctx.w(0.008))
        else:  # sunken zombie eyes
            ctx.E([exx - 0.011, hy - 0.008, exx + 0.011, hy + 0.012], fill=(30, 28, 24, 255))
    # mouth: ragged panting (feral) or slack (zed)
    if cfg.get("living"):
        ctx.E([cx - 0.020, hy + hr * 0.52, cx + 0.016, hy + hr * 0.80], fill=(74, 40, 34, 255),
              outline=OUTLINE, width=ctx.w(0.008))
    else:
        ctx.L([(cx - hr * 0.3, hy + hr * 0.55), (cx + hr * 0.22, hy + hr * 0.62)],
              skin_dk, ctx.w(0.012))
    if cfg.get("veins"):  # blackened vein web on neck and face
        for pts in ([(cx - 0.03, hy + hr * 0.9), (cx - 0.045, hy + hr * 1.25), (cx - 0.03, sh_y)],
                    [(cx + 0.035, hy + hr * 0.9), (cx + 0.05, hy + hr * 1.2), (cx + 0.045, sh_y)],
                    [(cx - hr * 0.55, hy - 0.01), (cx - hr * 0.30, hy + 0.015), (cx - hr * 0.42, hy + 0.05)],
                    [(cx + hr * 0.5, hy + 0.02), (cx + hr * 0.28, hy - 0.01)]):
            ctx.L(pts, VEIN, ctx.w(0.010))
    if cfg.get("tattoo"):  # Marine forearm tattoo
        ctx.L([(cx - sh - 0.045, sh_y + 0.16), (cx - sh - 0.028, sh_y + 0.24)],
              (70, 84, 120, 255), ctx.w(0.014))
    if cfg.get("sheen"):  # plaguebearer: glistening fluids
        for _ in range(7):
            x = rng.uniform(cx - sh, cx + sh); y = rng.uniform(sh_y, hip_y + 0.05)
            r = rng.uniform(0.010, 0.022)
            ctx.E([x - r, y - r * 0.6, x + r, y + r * 0.6], fill=(226, 232, 190, 255))
        for _ in range(3):  # drips
            x = rng.uniform(cx - wa * 0.8, cx + wa * 0.8)
            ctx.L([(x, hip_y + 0.02), (x, hip_y + 0.075)], (196, 204, 140, 255), ctx.w(0.011))
    if cfg.get("fungal"):  # cop: armor overgrown with fungal matter
        sprout_cluster(ctx, cx - sh * 0.45, sh_y + 0.10, 0.045, rng, n=10)
        sprout_cluster(ctx, cx + sh * 0.40, sh_y + 0.20, 0.040, rng, n=9)
        sprout_cluster(ctx, cx + 0.01, hip_y - 0.08, 0.035, rng, n=7)
        stalk(ctx, cx - sh * 0.62, sh_y + 0.03, h=0.075, lean=-0.02, cap_r=0.024)
        stalk(ctx, cx + sh * 0.58, sh_y + 0.06, h=0.06, lean=0.025, cap_r=0.020)
    if cfg.get("dark"):  # anaximander: melts into shadow
        ov = Image.new("RGBA", (size, size), (0, 0, 0, 0))
        od = ImageDraw.Draw(ov)
        s = size
        for box, a in (((0.06, 0.55, 0.94, 1.0), 120), ((0.16, 0.30, 0.84, 0.75), 70)):
            od.ellipse([box[0] * s, box[1] * s, box[2] * s, box[3] * s], fill=(8, 8, 14, a))
        ctx.img.alpha_composite(ov)
        ctx.d = ImageDraw.Draw(ctx.img)
        for sgn in (-1, 1):  # eyes still catch the light
            exx = cx + sgn * hr * 0.37
            ctx.E([exx - 0.007, hy - 0.006, exx + 0.007, hy + 0.008], fill=(214, 220, 200, 255))
    return ctx.img

def ocular(size, mid):
    """Regular zombie whose head is filled to the brim with eyes."""
    cfg = dict(skin=(150, 148, 138, 255), skin_hi=(178, 176, 164, 255),
               skin_dk=(112, 110, 100, 255), shirt=(70, 66, 74, 255),
               shirt_dk=(50, 47, 54, 255), pants=(50, 52, 58, 255), hunch=0.5)
    img = humanoid(size, {**cfg, "living": False}, mid)
    ctx = Ctx(size)
    ctx.img = img
    rng = rng_for(mid + "eyes")
    cx, hy, hr = 0.5, 0.175 + 0.5 * 0.055, 0.082
    eyes = [(-0.45, -0.45, 0.020), (0.05, -0.62, 0.024), (0.52, -0.40, 0.018),
            (-0.62, 0.02, 0.022), (-0.10, -0.10, 0.028), (0.38, 0.08, 0.026),
            (0.68, 0.28, 0.016), (-0.30, 0.38, 0.024), (0.12, 0.45, 0.020),
            (-0.58, -0.62, 0.014), (0.55, 0.55, 0.015), (0.0, 0.68, 0.013)]
    for i, (fx, fy, r) in enumerate(eyes):
        ex, ey = cx + fx * hr, hy + fy * hr
        ctx.E([ex - r, ey - r * 0.85, ex + r, ey + r * 0.85], fill=(228, 224, 210, 255),
              outline=OUTLINE, width=ctx.w(0.007))
        if i % 2 == 0:  # half swivel frantically: pupils off-center
            px, py = ex + rng.uniform(-0.4, 0.4) * r, ey + rng.uniform(-0.3, 0.3) * r
        else:           # the other half stare directly at you
            px, py = ex, ey
        pr = r * 0.42
        ctx.E([px - pr, py - pr, px + pr, py + pr], fill=(22, 20, 20, 255))
    return ctx.img

def crawler(size, mid):
    """Decayed grasper: ruined body crawling with eerily fluid reach."""
    rng = rng_for(mid)
    ctx = Ctx(size)
    cx = 0.5
    skin, skin_dk = (156, 166, 104, 255), (116, 126, 76, 255)
    shirt, shirt_dk = (104, 110, 74, 255), (78, 84, 54, 255)
    pants = (66, 68, 60, 255)
    # trailing decayed legs, one partly dissolved
    for sgn in (-1, 1):
        ctx.P([(cx + sgn * 0.10, 0.66), (cx + sgn * 0.20, 0.66),
               (cx + sgn * 0.31, 0.90), (cx + sgn * 0.17, 0.94)],
              fill=pants, outline=OUTLINE)
        ctx.E([cx + sgn * 0.235 - 0.03, 0.885, cx + sgn * 0.235 + 0.045, 0.945],
              fill=GORE_DK, outline=OUTLINE, width=ctx.w(0.01))
    # leaning torso, torn and stained
    ctx.P([(cx - 0.20, 0.30), (cx + 0.20, 0.30), (cx + 0.13, 0.68), (cx - 0.13, 0.68)],
          fill=shirt, outline=OUTLINE)
    ctx.P([(cx - 0.16, 0.32), (cx - 0.04, 0.32), (cx - 0.05, 0.62), (cx - 0.11, 0.62)],
          fill=shirt_dk)
    for _ in range(5):  # putrescent patches
        x = rng.uniform(cx - 0.16, cx + 0.16); y = rng.uniform(0.34, 0.62)
        r = rng.uniform(0.013, 0.026)
        ctx.E([x - r, y - r, x + r, y + r], fill=GORE_DK)
    # long reaching arms planted far forward
    for sgn in (-1, 1):
        ctx.P([(cx + sgn * 0.17, 0.32), (cx + sgn * 0.245, 0.32),
               (cx + sgn * 0.335, 0.82), (cx + sgn * 0.255, 0.85)],
              fill=shirt_dk if sgn > 0 else shirt, outline=OUTLINE)
        # grasping hand, fingers spread
        hx, hyy = cx + sgn * 0.295, 0.845
        ctx.E([hx - 0.030, hyy - 0.014, hx + 0.030, hyy + 0.026], fill=skin,
              outline=OUTLINE, width=ctx.w(0.01))
        for f in (-1, 0, 1):
            ctx.L([(hx + f * 0.016, hyy + 0.018), (hx + f * 0.026, hyy + 0.062)],
                  skin_dk, ctx.w(0.011))
    # low head
    ctx.E([cx - 0.085, 0.115, cx + 0.085, 0.285], fill=skin, outline=OUTLINE)
    ctx.E([cx - 0.05, 0.13, cx + 0.01, 0.20], fill=(182, 192, 128, 255))
    for sgn in (-1, 1):
        ctx.E([cx + sgn * 0.031 - 0.011, 0.185, cx + sgn * 0.031 + 0.011, 0.205],
              fill=(30, 28, 24, 255))
    ctx.L([(cx - 0.024, 0.245), (cx + 0.020, 0.252)], skin_dk, ctx.w(0.011))
    return ctx.img

def quadruped_red(size, final, mid):
    """Regenerating hunter (final=False) / abomination (final=True):
    all-fours figure of exposed, restitching red muscle."""
    rng = rng_for(mid)
    ctx = Ctx(size)
    muscle = (158, 62, 52, 255); muscle_hi = (188, 92, 78, 255); muscle_dk = (116, 40, 36, 255)
    sinew = (206, 168, 140, 255)
    # legs: front pair planted, rear pair crouched; final gets wrong joints
    legs = [(0.20, 0.52, 0.16, 0.94), (0.34, 0.55, 0.33, 0.95),
            (0.66, 0.55, 0.70, 0.95), (0.80, 0.52, 0.86, 0.94)]
    for i, (x0, y0, x1, y1) in enumerate(legs):
        bend = 0.05 if (final and i in (1, 2)) else 0.018
        mx, my = (x0 + x1) / 2 + (bend if i % 2 else -bend), (y0 + y1) / 2
        ctx.L([(x0, y0), (mx, my), (x1, y1)], OUTLINE, ctx.w(0.062))
        ctx.L([(x0, y0), (mx, my), (x1, y1)], muscle_dk if i % 2 else muscle, ctx.w(0.042))
        ctx.E([x1 - 0.030, y1 - 0.016, x1 + 0.036, y1 + 0.012], fill=muscle_dk, outline=OUTLINE,
              width=ctx.w(0.008))
        if final:  # exposed bone segment mid-limb
            ctx.L([(mx, my), (mx + 0.012, my + 0.07)], BONE, ctx.w(0.020))
    # long low body
    ctx.E([0.13, 0.28, 0.87, 0.58], fill=muscle, outline=OUTLINE)
    ctx.E([0.20, 0.31, 0.62, 0.46], fill=muscle_hi)
    # churning muscle striations + restitch seams
    for _ in range(9):
        x = rng.uniform(0.20, 0.78); y = rng.uniform(0.33, 0.52)
        ctx.L([(x, y), (x + rng.uniform(0.04, 0.09), y + rng.uniform(-0.02, 0.03))],
              muscle_dk, ctx.w(0.013))
    for _ in range(4):  # seam stitches
        x = rng.uniform(0.24, 0.72); y = rng.uniform(0.34, 0.50)
        for k in range(3):
            ctx.L([(x + k * 0.018, y - 0.010), (x + k * 0.018 + 0.008, y + 0.012)],
                  sinew, ctx.w(0.008))
    if final:  # dismantled skeleton showing through: rib arcs + spine knobs
        for i in range(4):
            x = 0.34 + i * 0.10
            ctx.P([(x, 0.30), (x + 0.035, 0.30), (x + 0.022, 0.52), (x - 0.006, 0.52)],
                  fill=BONE_DK)
        for i in range(5):
            x = 0.26 + i * 0.115
            ctx.E([x - 0.016, 0.265, x + 0.016, 0.295], fill=BONE, outline=OUTLINE,
                  width=ctx.w(0.007))
    # head low at front, jaw slack
    ctx.E([0.76, 0.34, 0.985, 0.58], fill=muscle, outline=OUTLINE)
    ctx.E([0.80, 0.37, 0.92, 0.46], fill=muscle_hi)
    ctx.P([(0.90, 0.52), (0.985, 0.545), (0.90, 0.60)], fill=muscle_dk, outline=OUTLINE)
    for k in range(3):  # teeth
        x = 0.915 + k * 0.020
        ctx.P([(x, 0.545), (x + 0.008, 0.545), (x + 0.004, 0.575)], fill=BONE_HI)
    ctx.E([0.845, 0.415, 0.872, 0.442], fill=(26, 22, 20, 255))  # eye
    if final:  # a second, wrongly-placed eye on the shoulder
        ctx.E([0.235, 0.335, 0.262, 0.362], fill=(232, 226, 210, 255), outline=OUTLINE,
              width=ctx.w(0.007))
        ctx.E([0.243, 0.343, 0.254, 0.354], fill=(22, 20, 20, 255))
    return ctx.img

def fursuit(size, mid):
    """Zombie in a torn anthropomorphic-animal cloth suit, caked in goo."""
    rng = rng_for(mid)
    ctx = Ctx(size)
    suit = (198, 206, 182, 255); suit_dk = (158, 168, 140, 255); suit_hi = (226, 232, 210, 255)
    cx = 0.5
    # legs + feet (padded suit feet)
    for sgn in (-1, 1):
        lx = cx + sgn * 0.065
        ctx.P([(lx - 0.048, 0.60), (lx + 0.048, 0.60), (lx + 0.05, 0.90), (lx - 0.05, 0.90)],
              fill=suit_dk, outline=OUTLINE)
        ctx.E([lx - 0.062, 0.885, lx + 0.068, 0.945], fill=suit, outline=OUTLINE,
              width=ctx.w(0.01))
    # chubby suit torso with pale belly patch
    ctx.E([cx - 0.185, 0.30, cx + 0.185, 0.66], fill=suit, outline=OUTLINE)
    ctx.E([cx - 0.105, 0.40, cx + 0.105, 0.635], fill=suit_hi)
    # arms, one hanging paw
    for sgn in (-1, 1):
        sx = cx + sgn * 0.165
        ctx.P([(sx - 0.030, 0.33), (sx + 0.030, 0.33), (sx + sgn * 0.045 + 0.026, 0.66),
               (sx + sgn * 0.045 - 0.026, 0.66)], fill=suit_dk if sgn > 0 else suit,
              outline=OUTLINE)
        ctx.E([sx + sgn * 0.045 - 0.034, 0.645, sx + sgn * 0.045 + 0.034, 0.705],
              fill=suit, outline=OUTLINE, width=ctx.w(0.01))  # paw
    # mascot head: round, ears, snout
    for sgn in (-1, 1):  # ears, one flopped
        if sgn < 0:
            ctx.E([cx - 0.135, 0.045, cx - 0.055, 0.125], fill=suit_dk, outline=OUTLINE)
            ctx.E([cx - 0.118, 0.062, cx - 0.072, 0.108], fill=(188, 148, 148, 255))
        else:
            ctx.P([(cx + 0.075, 0.115), (cx + 0.145, 0.085), (cx + 0.135, 0.165)],
                  fill=suit_dk, outline=OUTLINE)
    ctx.E([cx - 0.115, 0.09, cx + 0.115, 0.32], fill=suit, outline=OUTLINE)
    ctx.E([cx - 0.075, 0.105, cx + 0.01, 0.19], fill=suit_hi)
    ctx.E([cx - 0.048, 0.225, cx + 0.048, 0.295], fill=suit_hi, outline=OUTLINE,
          width=ctx.w(0.009))  # snout
    ctx.E([cx - 0.016, 0.232, cx + 0.016, 0.256], fill=(52, 44, 44, 255))  # nose
    ctx.L([(cx, 0.256), (cx, 0.272)], suit_dk, ctx.w(0.010))
    for sgn in (-1, 1):  # dead eyes behind the head's eye holes
        exx = cx + sgn * 0.052
        ctx.E([exx - 0.017, 0.155, exx + 0.017, 0.185], fill=(30, 28, 26, 255), outline=OUTLINE,
              width=ctx.w(0.007))
        ctx.E([exx - 0.005, 0.163, exx + 0.005, 0.176], fill=(120, 128, 104, 255))
    # torn patch: zombie skin and gore through the rip
    ctx.P([(cx + 0.055, 0.44), (cx + 0.125, 0.475), (cx + 0.10, 0.55), (cx + 0.045, 0.52)],
          fill=ZSKIN_DK, outline=OUTLINE)
    ctx.E([cx + 0.062, 0.475, cx + 0.105, 0.515], fill=GORE)
    # black goo and blood caking
    for _ in range(8):
        x = rng.uniform(cx - 0.17, cx + 0.17); y = rng.uniform(0.14, 0.90)
        r = rng.uniform(0.012, 0.030)
        ctx.E([x - r, y - r * 0.8, x + r, y + r * 0.8],
              fill=(26, 24, 22, 255) if rng.random() < 0.65 else GORE_DK)
    ctx.L([(cx - 0.03, 0.295), (cx - 0.022, 0.36)], (26, 24, 22, 255), ctx.w(0.014))  # goo drip from snout
    return ctx.img

# ------------------------------------------------------------------ frogs
def frog(size, cfg, mid):
    """Front-facing squat frog/toad. cfg: body/hi/dk, belly, warts (0..2),
    wart_sludge, eyes ('normal'|'pale'|'dead'|'mask'), mouth ('line'|'wail'),
    sludge (mouth drips), cracks, wraps, folds, smoke, acid, shadow,
    flat (shadow leaper profile), lean (treefrog)."""
    rng = rng_for(mid)
    ctx = Ctx(size)
    body = cfg["body"]; body_hi = cfg["hi"]; body_dk = cfg["dk"]
    belly = cfg.get("belly", body_hi)
    flat = cfg.get("flat")
    top = 0.30 if flat else 0.20
    bot = 0.92
    # hind legs: folded thighs at the sides + feet
    for sgn in (-1, 1):
        ctx.E([0.5 + sgn * 0.30 - 0.155, 0.52, 0.5 + sgn * 0.30 + 0.155, 0.94],
              fill=body_dk, outline=OUTLINE)
        ctx.E([0.5 + sgn * 0.30 - 0.10, 0.56, 0.5 + sgn * 0.30 + 0.06, 0.78], fill=body)
        fx = 0.5 + sgn * 0.335
        for t in (-1, 0, 1):  # toes
            ctx.L([(fx, 0.915), (fx + sgn * 0.045 + t * 0.012, 0.955)], body_dk, ctx.w(0.016))
    # main body mass
    ctx.E([0.13, top, 0.87, bot], fill=body, outline=OUTLINE)
    ctx.E([0.20, top + 0.05, 0.62, top + 0.42], fill=body_hi)  # back highlight
    ctx.E([0.30, 0.55, 0.70, 0.90], fill=belly)                # throat/belly
    if cfg.get("lean"):  # treefrog: toe pads on front feet, slimmer arms
        pass
    # front arms
    for sgn in (-1, 1):
        ax = 0.5 + sgn * 0.235
        ctx.P([(ax - 0.030, 0.55), (ax + 0.030, 0.55), (ax + sgn * 0.02 + 0.026, 0.90),
               (ax + sgn * 0.02 - 0.026, 0.90)], fill=body_dk, outline=OUTLINE)
        if cfg.get("lean"):  # round toe pads
            for t in (-1, 0, 1):
                tx = ax + sgn * 0.02 + t * 0.030
                ctx.E([tx - 0.013, 0.885, tx + 0.013, 0.912], fill=body_hi, outline=OUTLINE,
                      width=ctx.w(0.007))
    # warts over the back
    warts = cfg.get("warts", 0)
    if warts:
        n = 14 if warts == 2 else 8
        for _ in range(n):
            x = rng.uniform(0.22, 0.78); y = rng.uniform(top + 0.06, 0.55)
            r = rng.uniform(0.016, 0.034) * (1.25 if warts == 2 else 1.0)
            ctx.E([x - r, y - r * 0.8, x + r, y + r * 0.8], fill=body_dk, outline=OUTLINE,
                  width=ctx.w(0.008))
            ctx.E([x - r * 0.45, y - r * 0.5, x + r * 0.1, y], fill=body_hi)
            if cfg.get("wart_sludge") and rng.random() < 0.55:  # sludge-filled, fit to burst
                ctx.E([x - r * 0.4, y - r * 0.35, x + r * 0.4, y + r * 0.3],
                      fill=(196, 168, 74, 255))
    if cfg.get("osteo"):  # bone toad: warts stiffened into pale osteoderms
        for _ in range(9):
            x = rng.uniform(0.24, 0.76); y = rng.uniform(top + 0.05, 0.50)
            r = rng.uniform(0.016, 0.028)
            ctx.E([x - r, y - r * 0.8, x + r, y + r * 0.8], fill=BONE_DK, outline=OUTLINE,
                  width=ctx.w(0.008))
            ctx.E([x - r * 0.4, y - r * 0.45, x + r * 0.1, y], fill=BONE_HI)
    # eye bumps + eyes
    eyes = cfg.get("eyes", "normal")
    for sgn in (-1, 1):
        ex = 0.5 + sgn * 0.185
        ey = top + 0.045
        ctx.E([ex - 0.062, ey - 0.052, ex + 0.062, ey + 0.052], fill=body, outline=OUTLINE)
        if eyes == "pale":  # empty milky eyes, swivelled outward
            ctx.E([ex - 0.034, ey - 0.030, ex + 0.034, ey + 0.030], fill=(216, 218, 200, 255),
                  outline=OUTLINE, width=ctx.w(0.008))
            ctx.E([ex + sgn * 0.012 - 0.008, ey - 0.008, ex + sgn * 0.012 + 0.008, ey + 0.008],
                  fill=(120, 124, 110, 255))
        elif eyes == "dead":
            ctx.E([ex - 0.032, ey - 0.028, ex + 0.032, ey + 0.028], fill=(148, 146, 134, 255),
                  outline=OUTLINE, width=ctx.w(0.008))
            ctx.E([ex - 0.007, ey - 0.007, ex + 0.007, ey + 0.007], fill=(70, 68, 60, 255))
        elif eyes != "mask":
            ctx.E([ex - 0.034, ey - 0.030, ex + 0.034, ey + 0.030], fill=(206, 178, 84, 255),
                  outline=OUTLINE, width=ctx.w(0.008))
            ctx.E([ex - 0.009, ey - 0.026, ex + 0.009, ey + 0.026], fill=(24, 22, 18, 255))
    if eyes == "mask":  # death mask: ossified plate over the face + jagged beak
        ctx.P([(0.28, top + 0.16), (0.34, top - 0.015), (0.66, top - 0.015), (0.72, top + 0.16),
               (0.60, top + 0.30), (0.40, top + 0.30)], fill=BONE, outline=OUTLINE)
        ctx.P([(0.36, top + 0.03), (0.50, top - 0.005), (0.44, top + 0.10)], fill=BONE_HI)
        for sgn in (-1, 1):  # dead eyes in the mask's sockets
            ex = 0.5 + sgn * 0.115
            ctx.E([ex - 0.030, top + 0.045, ex + 0.030, top + 0.105], fill=(40, 38, 34, 255),
                  outline=OUTLINE, width=ctx.w(0.007))
            ctx.E([ex - 0.007, top + 0.068, ex + 0.007, top + 0.082], fill=(150, 148, 136, 255))
        # beak: upper plate points down over the mouth, jagged lower edge
        ctx.P([(0.40, top + 0.28), (0.60, top + 0.28), (0.565, top + 0.375), (0.52, top + 0.325),
               (0.485, top + 0.385), (0.45, top + 0.33), (0.425, top + 0.375)],
              fill=BONE_DK, outline=OUTLINE)
    # mouth
    my = top + 0.30
    if cfg.get("mouth") == "wail":  # frog dad: open unholy wail
        ctx.E([0.36, my - 0.015, 0.64, my + 0.115], fill=(52, 26, 24, 255), outline=OUTLINE)
        ctx.E([0.42, my + 0.045, 0.58, my + 0.105], fill=(128, 58, 52, 255))
        ctx.L([(0.30, my + 0.01), (0.36, my + 0.035)], body_dk, ctx.w(0.014))
        ctx.L([(0.70, my + 0.01), (0.64, my + 0.035)], body_dk, ctx.w(0.014))
    elif eyes != "mask":
        ctx.L([(0.31, my), (0.42, my + 0.028), (0.58, my + 0.028), (0.69, my)],
              body_dk, ctx.w(0.016))
    if cfg.get("sludge"):  # yellow-brown sludge dribbling from the mouth
        for x, ln in ((0.44, 0.10), (0.52, 0.16), (0.585, 0.08)):
            ctx.L([(x, my + 0.03), (x + 0.006, my + 0.03 + ln)], (172, 142, 62, 255), ctx.w(0.020))
            ctx.E([x - 0.012, my + 0.03 + ln - 0.012, x + 0.018, my + 0.03 + ln + 0.014],
                  fill=(196, 168, 74, 255))
        ctx.E([0.40, 0.90, 0.62, 0.955], fill=(160, 130, 58, 255))  # seeped pool
    if cfg.get("acid"):  # gastro bufo: acid leaking from the skin
        for _ in range(6):
            x = rng.uniform(0.24, 0.76); y = rng.uniform(top + 0.10, 0.62)
            ctx.L([(x, y), (x + 0.004, y + rng.uniform(0.05, 0.11))], (140, 196, 62, 255),
                  ctx.w(0.013))
        ctx.E([0.30, 0.905, 0.48, 0.955], fill=(140, 196, 62, 255))
        ctx.E([0.56, 0.91, 0.70, 0.952], fill=(140, 196, 62, 255))
        for _ in range(4):  # wet sheen
            x = rng.uniform(0.26, 0.70); y = rng.uniform(top + 0.06, 0.40)
            r = rng.uniform(0.010, 0.02)
            ctx.E([x - r, y - r * 0.6, x + r, y + r * 0.6], fill=(210, 232, 170, 255))
    if cfg.get("arcs"):  # shocker: bolts skipping across skin, arcing at the lips
        ov = Image.new("RGBA", (size, size), (0, 0, 0, 0))
        od = ImageDraw.Draw(ov)
        s = size
        pts_list = [[(0.24, 0.30), (0.33, 0.38), (0.30, 0.44), (0.40, 0.50)],
                    [(0.72, 0.26), (0.64, 0.35), (0.69, 0.42), (0.60, 0.47)],
                    [(0.40, my + 0.02), (0.46, my - 0.03), (0.52, my + 0.03), (0.58, my - 0.02)],
                    [(0.30, 0.62), (0.38, 0.68), (0.35, 0.74)]]
        for pts in pts_list:
            od.line([(x * s, y * s) for x, y in pts], fill=(240, 252, 255, 190),
                    width=max(1, round(s / 32)))
            od.line([(x * s, y * s) for x, y in pts], fill=(170, 230, 240, 110),
                    width=max(1, round(s / 16)))
        ctx.img.alpha_composite(ov)
        ctx.d = ImageDraw.Draw(ctx.img)
    if cfg.get("cracks"):  # husk: dry cracked skin + tainted dust wisps
        for _ in range(9):
            x = rng.uniform(0.24, 0.74); y = rng.uniform(top + 0.08, 0.66)
            ctx.L([(x, y), (x + rng.uniform(-0.05, 0.05), y + rng.uniform(0.02, 0.06)),
                   (x + rng.uniform(-0.08, 0.08), y + rng.uniform(0.07, 0.11))],
                  body_dk, ctx.w(0.009))
        ov = Image.new("RGBA", (size, size), (0, 0, 0, 0))
        od = ImageDraw.Draw(ov)
        s = size
        for _ in range(10):
            x = rng.uniform(0.15, 0.85); y = rng.uniform(0.15, 0.85)
            r = rng.uniform(0.008, 0.02)
            od.ellipse([(x - r) * s, (y - r) * s, (x + r) * s, (y + r) * s],
                       fill=(206, 192, 160, 110))
        ctx.img.alpha_composite(ov)
        ctx.d = ImageDraw.Draw(ctx.img)
    if cfg.get("wraps"):  # mummy: papery skin hugging bone, wrap bands
        for y in (0.38, 0.52, 0.66):
            ctx.L([(0.20, y), (0.80, y - 0.03)], (196, 182, 152, 255), ctx.w(0.020))
            ctx.L([(0.20, y + 0.022), (0.80, y - 0.008)], body_dk, ctx.w(0.008))
        ov = Image.new("RGBA", (size, size), (0, 0, 0, 0))  # shed ash at the base
        od = ImageDraw.Draw(ov)
        s = size
        for _ in range(8):
            x = rng.uniform(0.12, 0.88); y = rng.uniform(0.86, 0.97)
            r = rng.uniform(0.012, 0.03)
            od.ellipse([(x - r) * s, (y - r * 0.5) * s, (x + r) * s, (y + r * 0.5) * s],
                       fill=(168, 164, 156, 120))
        ctx.img.alpha_composite(ov)
        ctx.d = ImageDraw.Draw(ctx.img)
    if cfg.get("folds"):  # smoker hulk: crinkled dry folds
        for _ in range(8):
            x = rng.uniform(0.22, 0.72); y = rng.uniform(top + 0.10, 0.70)
            ctx.L([(x, y), (x + rng.uniform(0.03, 0.08), y + rng.uniform(0.01, 0.04)),
                   (x + rng.uniform(0.0, 0.04), y + rng.uniform(0.05, 0.09))],
                  body_dk, ctx.w(0.011))
    if cfg.get("smoke"):  # smoker hulk: constant haze fuming off the body
        ov = Image.new("RGBA", (size, size), (0, 0, 0, 0))
        od = ImageDraw.Draw(ov)
        s = size
        for _ in range(9):
            x = rng.uniform(0.16, 0.84); y = rng.uniform(0.06, 0.60)
            r = rng.uniform(0.05, 0.11)
            od.ellipse([(x - r) * s, (y - r * 0.7) * s, (x + r) * s, (y + r * 0.7) * s],
                       fill=(148, 148, 146, 84))
        ctx.img.alpha_composite(ov)
        ctx.d = ImageDraw.Draw(ctx.img)
    if cfg.get("shadow"):  # shadow leaper: darkness wreathes the ground around it
        ov = Image.new("RGBA", (size, size), (0, 0, 0, 0))
        od = ImageDraw.Draw(ov)
        s = size
        for box, a in (((0.02, 0.62, 0.98, 1.0), 130), ((0.10, 0.45, 0.90, 0.85), 70),
                       ((0.30, 0.10, 0.70, 0.35), 55)):
            od.ellipse([box[0] * s, box[1] * s, box[2] * s, box[3] * s], fill=(6, 8, 8, a))
        ctx.img.alpha_composite(ov)
        ctx.d = ImageDraw.Draw(ctx.img)
        for sgn in (-1, 1):  # eyes gleam out of the dark
            ex = 0.5 + sgn * 0.185
            ctx.E([ex - 0.010, top + 0.032, ex + 0.010, top + 0.058], fill=(208, 224, 190, 255))
    return ctx.img

# ---------------------------------------------------------- amalgamations
def breather(size, hub, mid):
    """Featureless pink flesh sac, swelling as it breathes; hub is the
    larger origin mass with breather buds growing off it."""
    rng = rng_for(mid)
    ctx = Ctx(size)
    pink = (216, 148, 152, 255); pink_hi = (238, 182, 182, 255); pink_dk = (182, 108, 116, 255)
    vein = (168, 84, 96, 255)
    if hub:  # main mass + satellite buds
        ctx.E([0.16, 0.30, 0.84, 0.92], fill=pink, outline=OUTLINE)
        ctx.E([0.24, 0.36, 0.58, 0.62], fill=pink_hi)
        for bx, by, br in ((0.13, 0.62, 0.115), (0.80, 0.58, 0.10), (0.66, 0.80, 0.09),
                           (0.28, 0.82, 0.075)):
            ctx.E([bx - br, by - br * 0.85, bx + br, by + br * 0.85], fill=pink_dk if bx < 0.2 else pink,
                  outline=OUTLINE)
            ctx.E([bx - br * 0.5, by - br * 0.5, bx + br * 0.05, by], fill=pink_hi)
    else:
        ctx.E([0.20, 0.26, 0.80, 0.92], fill=pink, outline=OUTLINE)
        ctx.E([0.28, 0.33, 0.58, 0.58], fill=pink_hi)
        ctx.E([0.30, 0.66, 0.72, 0.90], fill=pink_dk)  # collapsed lower fold
    # ridges: concentric breathing folds
    for i in range(3):
        y = 0.44 + i * 0.13
        ctx.L([(0.26, y), (0.40, y + 0.025), (0.60, y + 0.025), (0.74, y)], pink_dk, ctx.w(0.013))
    # veined, moist
    for _ in range(6):
        x = rng.uniform(0.28, 0.68); y = rng.uniform(0.32, 0.70)
        ctx.L([(x, y), (x + rng.uniform(-0.06, 0.06), y + rng.uniform(0.04, 0.09))],
              vein, ctx.w(0.010))
    for _ in range(5):  # moist speculars
        x = rng.uniform(0.28, 0.66); y = rng.uniform(0.30, 0.55)
        r = rng.uniform(0.008, 0.016)
        ctx.E([x - r, y - r * 0.6, x + r, y + r * 0.6], fill=(250, 226, 224, 255))
    return ctx.img

def spotter(size, mid):
    """Inflated white body on five dainty legs; two limbs end in
    head-sized compound eyes."""
    rng = rng_for(mid)
    ctx = Ctx(size)
    white = (232, 230, 222, 255); white_dk = (192, 190, 180, 255)
    # five dainty legs
    for lx in (0.24, 0.38, 0.50, 0.62, 0.76):
        ctx.L([(lx, 0.62), (lx - 0.012, 0.80), (lx + 0.006, 0.94)], OUTLINE, ctx.w(0.026))
        ctx.L([(lx, 0.62), (lx - 0.012, 0.80), (lx + 0.006, 0.94)], white_dk, ctx.w(0.013))
    # eye limbs raised, waving
    for sgn, bend in ((-1, -0.05), (1, 0.07)):
        ex = 0.5 + sgn * 0.33; ey = 0.155
        ctx.L([(0.5 + sgn * 0.10, 0.36), (0.5 + sgn * 0.24, 0.26), (ex + bend * 0.3, ey + 0.07)],
              OUTLINE, ctx.w(0.034))
        ctx.L([(0.5 + sgn * 0.10, 0.36), (0.5 + sgn * 0.24, 0.26), (ex + bend * 0.3, ey + 0.07)],
              white_dk, ctx.w(0.018))
        # head-sized compound eye
        r = 0.105
        exx = ex + bend * 0.3
        ctx.E([exx - r, ey - r, exx + r, ey + r], fill=(196, 84, 60, 255), outline=OUTLINE)
        for _ in range(14):  # facets
            fx = rng.uniform(exx - r * 0.75, exx + r * 0.75); fy = rng.uniform(ey - r * 0.75, ey + r * 0.75)
            if (fx - exx) ** 2 + (fy - ey) ** 2 < (r * 0.8) ** 2:
                fr = rng.uniform(0.010, 0.020)
                ctx.E([fx - fr, fy - fr, fx + fr, fy + fr],
                      fill=(222, 120, 92, 255) if rng.random() < 0.5 else (168, 64, 48, 255))
        ctx.E([exx - r * 0.45, ey - r * 0.62, exx - r * 0.05, ey - r * 0.22],
              fill=(246, 190, 170, 255))
    # inflated body
    ctx.E([0.24, 0.30, 0.76, 0.70], fill=white, outline=OUTLINE)
    ctx.E([0.31, 0.35, 0.58, 0.52], fill=(246, 244, 238, 255))
    ctx.L([(0.34, 0.60), (0.66, 0.60)], white_dk, ctx.w(0.010))  # seam under the sac
    return ctx.img

def corroder(size, mid):
    """Gangly six-legger; green pustules at the limb bases feed veins to
    two blackened claws; a fused seventh leg forms a jagged proboscis."""
    rng = rng_for(mid)
    ctx = Ctx(size)
    hide = (188, 176, 96, 255); hide_dk = (152, 140, 66, 255); hide_hi = (216, 206, 128, 255)
    pust = (104, 168, 62, 255); pust_hi = (150, 204, 96, 255)
    # six gangly legs
    for i, lx in enumerate((0.14, 0.28, 0.42, 0.58, 0.72, 0.86)):
        top_y = 0.52
        knee = (lx + (0.03 if i % 2 else -0.03), 0.68)
        foot = (lx + (0.05 if i % 2 else -0.05), 0.95)
        ctx.L([(lx, top_y), knee, foot], OUTLINE, ctx.w(0.030))
        ctx.L([(lx, top_y), knee, foot], hide_dk, ctx.w(0.015))
        ctx.E([lx - 0.030, top_y - 0.028, lx + 0.030, top_y + 0.028], fill=pust,
              outline=OUTLINE, width=ctx.w(0.008))  # pustule at the base
        ctx.E([lx - 0.012, top_y - 0.014, lx + 0.008, top_y + 0.004], fill=pust_hi)
    # irregular body
    ctx.E([0.16, 0.30, 0.84, 0.58], fill=hide, outline=OUTLINE)
    ctx.E([0.24, 0.34, 0.56, 0.47], fill=hide_hi)
    for _ in range(4):  # more pustules along the back
        x = rng.uniform(0.26, 0.74); y = rng.uniform(0.33, 0.44)
        r = rng.uniform(0.016, 0.026)
        ctx.E([x - r, y - r, x + r, y + r], fill=pust, outline=OUTLINE, width=ctx.w(0.007))
    # big veins running to the two blackened claws (front, viewer-left/right)
    for sgn in (-1, 1):
        vx = 0.5 + sgn * 0.30
        ctx.L([(0.5 + sgn * 0.08, 0.44), (vx - sgn * 0.05, 0.52), (vx, 0.66)],
              pust, ctx.w(0.016))
        # claw arm + blackened claw
        ctx.L([(vx, 0.55), (vx + sgn * 0.03, 0.72)], OUTLINE, ctx.w(0.034))
        ctx.L([(vx, 0.55), (vx + sgn * 0.03, 0.72)], hide_dk, ctx.w(0.018))
        cxp, cyp = vx + sgn * 0.03, 0.74
        for f in (-1, 1):
            ctx.P([(cxp, cyp), (cxp + sgn * 0.012 + f * 0.020, cyp + 0.085),
                   (cxp + sgn * 0.030, cyp + 0.012)], fill=(30, 28, 26, 255), outline=OUTLINE)
    # head front-center: eyes over the fused proboscis
    ctx.E([0.40, 0.40, 0.60, 0.56], fill=hide_dk, outline=OUTLINE)
    for sgn in (-1, 1):
        ctx.E([0.5 + sgn * 0.045 - 0.011, 0.435, 0.5 + sgn * 0.045 + 0.011, 0.457],
              fill=(26, 24, 20, 255))
    # jagged proboscis below the eyes, dripping caustic
    pts = [(0.465, 0.545), (0.535, 0.545), (0.520, 0.60), (0.545, 0.615), (0.508, 0.635),
           (0.492, 0.60), (0.472, 0.625), (0.478, 0.585)]
    ctx.P(pts, fill=(52, 48, 40, 255), outline=OUTLINE)
    ctx.L([(0.50, 0.635), (0.50, 0.685)], (140, 196, 62, 255), ctx.w(0.013))
    ctx.E([0.488, 0.678, 0.512, 0.700], fill=(150, 204, 96, 255))
    return ctx.img

def jumper(size, mid):
    """Feline-gait quadruped; three limbs hold an almost-translucent
    membrane on its side/rear; raven beak with jagged black teeth."""
    rng = rng_for(mid)
    ctx = Ctx(size)
    hide = (226, 224, 216, 255); hide_dk = (186, 184, 174, 255)
    # membrane held out on the rear/side (translucent -> overlay)
    ov = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    od = ImageDraw.Draw(ov)
    s = size
    od.polygon([(0.16 * s, 0.30 * s), (0.05 * s, 0.52 * s), (0.13 * s, 0.74 * s),
                (0.34 * s, 0.66 * s), (0.36 * s, 0.38 * s)], fill=(214, 226, 230, 105))
    od.line([(0.34 * s, 0.40 * s), (0.07 * s, 0.52 * s)], fill=(190, 200, 204, 160),
            width=max(1, round(s / 32)))
    od.line([(0.35 * s, 0.55 * s), (0.13 * s, 0.72 * s)], fill=(190, 200, 204, 160),
            width=max(1, round(s / 32)))
    ctx.img.alpha_composite(ov)
    ctx.d = ImageDraw.Draw(ctx.img)
    # four legs, feline
    for lx, back in ((0.30, True), (0.42, True), (0.66, False), (0.78, False)):
        ctx.L([(lx, 0.55), (lx - 0.01, 0.76), (lx + 0.015, 0.93)], OUTLINE, ctx.w(0.040))
        ctx.L([(lx, 0.55), (lx - 0.01, 0.76), (lx + 0.015, 0.93)], hide_dk, ctx.w(0.022))
        ctx.E([lx - 0.005, 0.915, lx + 0.045, 0.945], fill=hide_dk, outline=OUTLINE,
              width=ctx.w(0.007))
    # body
    ctx.E([0.24, 0.30, 0.84, 0.60], fill=hide, outline=OUTLINE)
    ctx.E([0.32, 0.34, 0.64, 0.47], fill=(242, 240, 234, 255))
    # membrane strut limbs along the top edge
    for px, py in ((0.30, 0.33), (0.20, 0.50), (0.30, 0.62)):
        ctx.L([(0.38, 0.42), (px, py)], hide_dk, ctx.w(0.013))
        ctx.E([px - 0.011, py - 0.011, px + 0.011, py + 0.011], fill=hide_dk, outline=OUTLINE,
              width=ctx.w(0.006))
    # head: mostly raven beak
    ctx.E([0.76, 0.28, 0.94, 0.50], fill=hide, outline=OUTLINE)
    ctx.E([0.815, 0.335, 0.845, 0.365], fill=(26, 24, 20, 255))  # eye
    ctx.P([(0.90, 0.36), (0.995, 0.415), (0.90, 0.44)], fill=(40, 38, 44, 255), outline=OUTLINE)
    ctx.P([(0.90, 0.44), (0.975, 0.435), (0.905, 0.50)], fill=(58, 56, 64, 255), outline=OUTLINE)
    for k in range(3):  # jagged black teeth in the opened beak
        x = 0.915 + k * 0.020
        ctx.P([(x, 0.442), (x + 0.009, 0.442), (x + 0.004, 0.472)], fill=(18, 16, 16, 255))
    return ctx.img

def zapper(size, mid):
    """Crablike: six multi-jointed legs, white armor plates, a large
    glowing organ bulging between them, one spark-tipped raised limb."""
    rng = rng_for(mid)
    ctx = Ctx(size)
    plate = (228, 226, 218, 255); plate_dk = (188, 186, 176, 255)
    # six multi-jointed legs
    for sgn in (-1, 1):
        for i, off in enumerate((0.16, 0.24, 0.30)):
            hx = 0.5 + sgn * off * 0.55
            hy = 0.52 + i * 0.10
            knee = (0.5 + sgn * (off + 0.13), hy - 0.06)
            foot = (0.5 + sgn * (off + 0.20), 0.95)
            ctx.L([(hx, hy), knee, foot], OUTLINE, ctx.w(0.034))
            ctx.L([(hx, hy), knee, foot], plate_dk, ctx.w(0.018))
            ctx.E([knee[0] - 0.014, knee[1] - 0.014, knee[0] + 0.014, knee[1] + 0.014],
                  fill=plate, outline=OUTLINE, width=ctx.w(0.007))
    # armored shell: overlapping plates
    ctx.E([0.20, 0.34, 0.80, 0.72], fill=plate, outline=OUTLINE)
    for i, (x0, x1) in enumerate(((0.24, 0.44), (0.40, 0.62), (0.58, 0.78))):
        ctx.P([(x0, 0.40 + i * 0.02), (x1, 0.40 + i * 0.02), (x1 - 0.03, 0.68),
               (x0 + 0.03, 0.68)], fill=plate_dk if i == 1 else plate, outline=OUTLINE)
    ctx.E([0.27, 0.38, 0.52, 0.50], fill=(244, 242, 236, 255))
    # the glowing organ bulging between the plates
    ctx.E([0.385, 0.44, 0.615, 0.68], fill=(96, 216, 226, 255), outline=OUTLINE)
    ctx.E([0.42, 0.475, 0.58, 0.645], fill=(168, 240, 246, 255))
    ctx.E([0.455, 0.51, 0.545, 0.60], fill=(232, 252, 253, 255))
    for _ in range(5):  # charge motes in the organ
        x = rng.uniform(0.42, 0.58); y = rng.uniform(0.48, 0.63)
        r = rng.uniform(0.006, 0.012)
        ctx.E([x - r, y - r, x + r, y + r], fill=(240, 254, 255, 255))
    # raised appendage from the organ, tip ringed with sparks
    ctx.L([(0.56, 0.46), (0.66, 0.30), (0.70, 0.17)], OUTLINE, ctx.w(0.036))
    ctx.L([(0.56, 0.46), (0.66, 0.30), (0.70, 0.17)], plate_dk, ctx.w(0.019))
    tx, ty = 0.70, 0.155
    ctx.E([tx - 0.020, ty - 0.020, tx + 0.020, ty + 0.020], fill=(240, 252, 255, 255),
          outline=OUTLINE, width=ctx.w(0.007))
    for a in range(8):  # brilliant white sparks
        ang = a * math.pi / 4 + 0.3
        r0, r1 = 0.030, rng.uniform(0.048, 0.068)
        ctx.L([(tx + math.cos(ang) * r0, ty + math.sin(ang) * r0),
               (tx + math.cos(ang) * r1, ty + math.sin(ang) * r1)],
              (250, 253, 255, 255), ctx.w(0.010))
    # small eyes under the shell rim
    for sgn in (-1, 1):
        ctx.E([0.5 + sgn * 0.07 - 0.010, 0.665, 0.5 + sgn * 0.07 + 0.010, 0.685],
              fill=(26, 24, 20, 255))
    return ctx.img

def soldier(size, mid):
    """Bear-like armored bulk; clubbed tail; two talon limbs framing a
    vertical mouth on a smooth, featureless head."""
    rng = rng_for(mid)
    ctx = Ctx(size)
    armor = (92, 94, 100, 255); armor_dk = (62, 64, 70, 255); armor_hi = (124, 126, 134, 255)
    # clubbed tail sweeping from the rear
    ctx.L([(0.22, 0.55), (0.10, 0.66), (0.09, 0.82)], OUTLINE, ctx.w(0.075))
    ctx.L([(0.22, 0.55), (0.10, 0.66), (0.09, 0.82)], armor_dk, ctx.w(0.048))
    ctx.E([0.035, 0.78, 0.145, 0.90], fill=armor_dk, outline=OUTLINE)  # club
    ctx.E([0.055, 0.80, 0.10, 0.85], fill=armor)
    # four stout legs
    for lx in (0.26, 0.40, 0.62, 0.76):
        ctx.P([(lx - 0.045, 0.58), (lx + 0.045, 0.58), (lx + 0.04, 0.90), (lx - 0.04, 0.90)],
              fill=armor_dk, outline=OUTLINE)
        ctx.E([lx - 0.052, 0.885, lx + 0.058, 0.93], fill=(44, 45, 50, 255), outline=OUTLINE,
              width=ctx.w(0.008))
    # massive plated body
    ctx.E([0.14, 0.24, 0.88, 0.66], fill=armor, outline=OUTLINE)
    ctx.E([0.22, 0.28, 0.58, 0.44], fill=armor_hi)
    for i in range(3):  # plate seams
        x = 0.34 + i * 0.15
        ctx.L([(x, 0.27), (x - 0.02, 0.62)], armor_dk, ctx.w(0.013))
    for _ in range(5):  # dents and scoring
        x = rng.uniform(0.24, 0.80); y = rng.uniform(0.30, 0.55)
        ctx.L([(x, y), (x + rng.uniform(0.02, 0.05), y + rng.uniform(-0.01, 0.02))],
              armor_dk, ctx.w(0.010))
    # smooth featureless head with the vertical mouth
    ctx.E([0.70, 0.26, 0.965, 0.56], fill=armor, outline=OUTLINE)
    ctx.E([0.75, 0.30, 0.88, 0.40], fill=armor_hi)
    mx = 0.845
    ctx.L([(mx, 0.345), (mx, 0.50)], (16, 15, 16, 255), ctx.w(0.020))
    for k in range(3):  # teeth along the vertical mouth
        y = 0.375 + k * 0.045
        for sgn in (-1, 1):
            ctx.P([(mx + sgn * 0.012, y), (mx + sgn * 0.030, y + 0.010), (mx + sgn * 0.012, y + 0.020)],
                  fill=BONE_HI)
    # two short talon limbs framing the head
    for sgn, ax in ((-1, 0.665), (1, 0.975)):
        ctx.L([(ax - sgn * 0.03, 0.50), (ax, 0.62)], OUTLINE, ctx.w(0.042))
        ctx.L([(ax - sgn * 0.03, 0.50), (ax, 0.62)], armor_dk, ctx.w(0.024))
        for f in (-1, 0, 1):
            ctx.P([(ax, 0.615), (ax + f * 0.022 + sgn * 0.008, 0.685), (ax + sgn * 0.018, 0.63)],
                  fill=(22, 21, 22, 255), outline=OUTLINE)
    return ctx.img

# ------------------------------------------------------------------- jobs
FERAL_SKIN = dict(skin=HSKIN, skin_hi=HSKIN_HI, skin_dk=HSKIN_DK, living=True)
NAVY_U = dict(shirt=NAVY, shirt_dk=NAVY_DK, pants=(44, 48, 60, 255))

def HF(cfg):
    return lambda size, mid: humanoid(size, cfg, mid)

JOBS = [
    # --- feral humans (all 62.5 L -> 32) ---
    ("mon_feral_sailor_wrench", 32, HF({**FERAL_SKIN, **NAVY_U, "weapon": "wrench",
                                        "veins": True, "bloodshot": True, "hunch": 0.7})),
    ("mon_feral_sailor_axe", 32, HF({**FERAL_SKIN, **NAVY_U, "weapon": "axe",
                                     "veins": True, "bloodshot": True, "stains": True, "hunch": 0.75})),
    ("mon_feral_sailor_mop", 32, HF({**FERAL_SKIN, "shirt": (92, 104, 128, 255),
                                     "shirt_dk": (66, 76, 96, 255), "pants": (52, 56, 66, 255),
                                     "weapon": "mop", "bloodshot": True, "veins": True, "hunch": 0.65})),
    ("mon_feral_sailor_lug_wrench", 32, HF({**FERAL_SKIN, **NAVY_U, "weapon": "lug",
                                            "oil": True, "hair_wild": True, "bloodshot": True, "hunch": 0.7})),
    ("mon_feral_marine_bayonet", 32, HF({**FERAL_SKIN, "shirt": (76, 82, 54, 255),
                                         "shirt_dk": (54, 60, 38, 255), "pants": (60, 64, 44, 255),
                                         "weapon": "rifle", "stains": True, "tattoo": True,
                                         "bloodshot": True, "hunch": 0.6})),
    ("mon_feral_officer", 32, HF({**FERAL_SKIN, "shirt": (34, 48, 88, 255),
                                  "shirt_dk": (24, 34, 64, 255), "pants": (40, 44, 58, 255),
                                  "weapon": "pistol", "bloodshot": True, "hunch": 0.8})),
    ("mon_feral_cop_fungal_infected", 32, HF({**FERAL_SKIN, "shirt": (46, 66, 122, 255),
                                              "shirt_dk": (32, 46, 86, 255), "pants": (36, 40, 52, 255),
                                              "weapon": "baton", "vest": True, "fungal": True,
                                              "bloodshot": True, "hunch": 0.6})),
    ("mon_feral_prisoner", 32, HF({**FERAL_SKIN, "shirt": (112, 102, 90, 255),
                                   "shirt_dk": (84, 76, 66, 255), "pants": (70, 64, 56, 255),
                                   "weapon": "shiv", "stains": True, "bloodshot": True, "hunch": 0.75})),
    ("mon_feral_serum_user_locutus", 32, HF({"skin": HEALTHY, "skin_hi": HEALTHY_HI,
                                             "skin_dk": (182, 134, 96, 255), "living": True,
                                             "shirt": ORANGE, "shirt_dk": ORANGE_DK,
                                             "pants": ORANGE_DK, "weapon": "pipe", "hunch": 0.45})),
    ("mon_feral_serum_user_noonien", 32, HF({**FERAL_SKIN, "shirt": ORANGE, "shirt_dk": ORANGE_DK,
                                             "pants": ORANGE_DK, "weapon": "pipe",
                                             "bloodshot": True, "hunch": 0.55})),
    ("mon_feral_serum_user_armus", 32, HF({"skin": HEALTHY, "skin_hi": HEALTHY_HI,
                                           "skin_dk": (182, 134, 96, 255), "living": True,
                                           "shirt": ORANGE, "shirt_dk": ORANGE_DK,
                                           "pants": ORANGE_DK, "bloodshot": True, "hunch": 0.5})),
    ("mon_feral_serum_user_landru", 32, HF({**FERAL_SKIN, "shirt": ORANGE, "shirt_dk": ORANGE_DK,
                                            "pants": ORANGE_DK, "bloodshot": True, "blur": True,
                                            "hunch": 0.55})),
    ("mon_feral_serum_user_lore", 32, HF({**FERAL_SKIN, "shirt": ORANGE, "shirt_dk": ORANGE_DK,
                                          "pants": ORANGE_DK, "weapon": "spear", "cobbled": True,
                                          "bloodshot": True, "hunch": 0.4})),
    ("mon_feral_serum_user_anaximander", 32, HF({**FERAL_SKIN, "shirt": (150, 76, 24, 255),
                                                 "shirt_dk": (110, 54, 18, 255), "pants": (110, 54, 18, 255),
                                                 "bloodshot": True, "dark": True, "hunch": 0.6})),
    # --- zed_misc (all 62.5 L -> 32) ---
    ("mon_zombie_summoner", 32, HF(dict(skin=(168, 160, 148, 255), skin_hi=(196, 188, 172, 255),
                                        skin_dk=(128, 120, 108, 255), shirt=(40, 32, 36, 255),
                                        shirt_dk=(26, 20, 24, 255), pants=(34, 30, 34, 255),
                                        hunch=0.08))),
    ("mon_zombie_regenerating_hunter", 32, lambda s, m: quadruped_red(s, False, m)),
    ("mon_zombie_regenerating_final", 32, lambda s, m: quadruped_red(s, True, m)),
    ("mon_zombie_ocular", 32, ocular),
    ("mon_zombie_fursuit", 32, fursuit),
    ("mon_zombie_rot_grabber", 32, crawler),
    ("mon_zombie_rot_diseased", 32, HF(dict(skin=(168, 178, 110, 255), skin_hi=(196, 204, 140, 255),
                                            skin_dk=(128, 138, 80, 255), shirt=(96, 100, 66, 255),
                                            shirt_dk=(70, 74, 46, 255), pants=(58, 60, 48, 255),
                                            sheen=True, hunch=0.65))),
    # --- zed_amalgamation ---
    ("mon_amalgamation_breather", 32, lambda s, m: breather(s, False, m)),       # 62.5 L
    ("mon_amalgamation_breather_hub", 32, lambda s, m: breather(s, True, m)),    # 62.5 L
    ("mon_amalgamation_spotter", 32, spotter),                                    # 37 L
    ("mon_amalgamation_corroder", 32, corroder),                                  # 37 L
    ("mon_amalgamation_jumper", 32, jumper),                                      # 90 L
    ("mon_amalgamation_zapper", 64, zapper),                                      # 120 L
    ("mon_amalgamation_soldier", 64, soldier),                                    # 200 L
    # --- zed_amphibian ---
    ("mon_frog_husk", 32, lambda s, m: frog(s, dict(                          # 80 L
        body=(128, 116, 84, 255), hi=(156, 144, 108, 255), dk=(96, 86, 60, 255),
        belly=(148, 134, 100, 255), eyes="dead", cracks=True), m)),
    ("mon_frog_dad", 64, lambda s, m: frog(s, dict(                           # 200 L
        body=(116, 178, 84, 255), hi=(148, 204, 110, 255), dk=(84, 138, 58, 255),
        belly=(196, 220, 150, 255), mouth="wail"), m)),
    ("mon_toad_zombie", 64, lambda s, m: frog(s, dict(                         # 120 L
        body=(110, 116, 70, 255), hi=(140, 146, 96, 255), dk=(80, 86, 48, 255),
        belly=(160, 162, 118, 255), warts=1, eyes="dead"), m)),
    ("mon_frog_shocker", 64, lambda s, m: frog(s, dict(                        # 120 L
        body=(128, 188, 192, 255), hi=(168, 220, 222, 255), dk=(92, 148, 152, 255),
        belly=(196, 232, 230, 255), arcs=True), m)),
    ("mon_toad_bloat", 64, lambda s, m: frog(s, dict(                          # 120 L
        body=(56, 72, 46, 255), hi=(84, 102, 66, 255), dk=(36, 48, 30, 255),
        belly=(96, 110, 74, 255), warts=1, sludge=True), m)),
    ("mon_zombtreefrog", 64, lambda s, m: frog(s, dict(                        # 108.5 L
        body=(110, 116, 70, 255), hi=(142, 148, 96, 255), dk=(80, 86, 48, 255),
        belly=(168, 170, 122, 255), eyes="pale", lean=True), m)),
    ("mon_frog_shadow", 64, lambda s, m: frog(s, dict(                         # 108.5 L
        body=(66, 76, 58, 255), hi=(90, 100, 78, 255), dk=(44, 52, 38, 255),
        belly=(84, 94, 72, 255), flat=True, shadow=True), m)),
    ("mon_toad_wart", 64, lambda s, m: frog(s, dict(                           # 300 L
        body=(106, 112, 68, 255), hi=(136, 142, 94, 255), dk=(76, 82, 46, 255),
        belly=(156, 158, 114, 255), warts=2, wart_sludge=True, sludge=True), m)),
    ("mon_toad_bone", 64, lambda s, m: frog(s, dict(                           # 240 L
        body=(52, 64, 42, 255), hi=(80, 96, 62, 255), dk=(34, 44, 28, 255),
        belly=(92, 104, 70, 255), eyes="mask", osteo=True), m)),
    ("mon_frog_mummy", 64, lambda s, m: frog(s, dict(                          # 120 L
        body=(140, 126, 96, 255), hi=(168, 154, 122, 255), dk=(104, 92, 66, 255),
        belly=(158, 144, 114, 255), eyes="dead", wraps=True), m)),
    ("mon_frog_smoker_hulk", 64, lambda s, m: frog(s, dict(                    # 200 L
        body=(120, 110, 84, 255), hi=(148, 138, 108, 255), dk=(88, 80, 58, 255),
        belly=(140, 130, 102, 255), eyes="dead", folds=True, smoke=True), m)),
    ("mon_gastro_bufo", 64, lambda s, m: frog(s, dict(                         # 200 L
        body=(54, 70, 44, 255), hi=(84, 102, 64, 255), dk=(36, 48, 30, 255),
        belly=(96, 110, 72, 255), warts=1, acid=True), m)),
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

# contact sheet: every sprite over a neutral backdrop, 8 per row
cols = 8
rows = (len(JOBS) + cols - 1) // cols
cell = 64
sheet = Image.new("RGBA", (cols * cell, rows * cell), (58, 62, 54, 255))
for i, (mid, size, _draw) in enumerate(JOBS):
    p = os.path.join(BASE, SHEET[size], "monsters", mid, mid + ".png")
    im = Image.open(p)
    if im.width != cell:
        im = im.resize((cell, cell), Image.NEAREST)
    sheet.alpha_composite(im, ((i % cols) * cell, (i // cols) * cell))
sheet.save(os.path.join(BASE, "preview_ferals_zeds.png"))
print("wrote preview_ferals_zeds.png", sheet.size, "jobs:", len(JOBS))
