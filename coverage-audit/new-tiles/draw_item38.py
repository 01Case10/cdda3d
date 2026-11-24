#!/usr/bin/env python3
"""Batch ITEM38 (Phase 2g.4 scope extension): bludgeons.

39 ids from data/json/items/melee/bludgeons.json
(authoritative list embedded below; source list:
~/workspace/item-batches/38.txt). Copy-from chains were
resolved against the data before drawing.

Families:
- Macuahuitls: flat wooden paddle clubs with blades set
  along both edges -- hammered razors (scrap), stone
  chips, glass shards, teeth, and the steel-grade line
  lc/mc/hc/ch/qt (grade shading follows ITEM18: lc dull
  mild steel + bare grip end, mc medium + leather wrap,
  hc bright high steel + sturdy wrap, ch dark blue
  hardened steel + molded grip, qt brightest tempered
  steel + brass fittings).
- Battle axes: wooden haft, big single blade; grades as
  above (plain battleaxe resolves to the mc entry).
  _fake is a dull aluminum replica (blunt, no bright
  edge); _inferior is budget steel with the head seated
  askew and a visible socket gap.
- War hammers: hafted hammer face + back spike; grade
  line lc/hc/ch/qt, _fake a small pale aluminum replica,
  _inferior with a drooped face and a bent, dull spike.
- Polearms: crowbar_warhammer (bec de crowbar -- hammer
  face one side, forked crowbar flat the other) and
  lucern_hammerfake (lucerne hammer: face, top spike and
  hook, in cheap pale aluminum).
- Bats: bat_metal_bolt (aluminum barrel with bolts
  threaded through it), goedendag_bat (wooden bat, iron
  cap and a spike out the top).
- hammer_sledge_crude (homewrecker): irregular welded
  steel chunks on a wooden handle.
- Canes: makeshift (rough wood, crook handle), sinister
  (red plastic, shiny red orb in claws), wizard canes
  (white orb; off dim, on with an opaque pale halo --
  glow faked with opaque rings per the opacity rule;
  cheap is blue plastic with a smaller orb).
- Crutches (aluminum pair / wooden makeshift pair) drawn
  upright as a pair; hockey stick with black-taped blade;
  makeshift_sap as a flat stitched leather pouch;
  sharp_toothbrush (plastic shank) as a light-blue
  toothbrush handle ground to a point.

Renderer: hafted weapons are drawn in a local frame
(u along the weapon, v across it) rotated 45 degrees --
grip low-left, head high-right -- matching ITEM18/19.
"""

import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

BASE = os.path.dirname(os.path.abspath(__file__))
ITEMS_DIR = os.path.join(BASE, "pngs_normal_32x32", "items")
CREDITS = (
    "Drawn from the item's in-game description for the CDDA Voxel "
    "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.\n"
)

IDS = [
    "aztec_sword_scrap", "aztec_sword_stone", "bat_metal_bolt",
    "battleaxe", "battleaxe_fake", "battleaxe_inferior",
    "ch_aztec_sword", "ch_battleaxe", "ch_warhammer",
    "crowbar_warhammer", "crutches", "glass_macuahuitl",
    "goedendag_bat", "hammer_sledge_crude", "hc_aztec_sword",
    "hc_battleaxe", "hc_warhammer", "hockey_stick",
    "lc_aztec_sword", "lc_battleaxe", "lc_warhammer",
    "lucern_hammerfake", "makeshift_cane", "makeshift_crutches",
    "makeshift_sap", "mc_aztec_sword", "mc_battleaxe",
    "qt_aztec_sword", "qt_battleaxe", "qt_warhammer",
    "sharp_toothbrush", "sinister_cane", "teeth_macuahuitl",
    "warhammer_fake", "warhammer_inferior", "wizard_cane",
    "wizard_cane_cheap", "wizard_cane_cheap_on", "wizard_cane_on",
]

# ---------------------------------------------------------------- palettes
# Steel grades, following ITEM18's shading (base, highlight, dark).
STEEL = {
    "lc":     dict(base=(148, 150, 157), hi=(181, 183, 189), dark=(96, 98, 105)),
    "mc":     dict(base=(176, 180, 188), hi=(215, 220, 228), dark=(112, 116, 126)),
    "hc":     dict(base=(197, 202, 212), hi=(239, 243, 251), dark=(128, 134, 146)),
    "ch":     dict(base=(129, 141, 163), hi=(190, 202, 224), dark=(82, 92, 112)),
    "qt":     dict(base=(204, 212, 227), hi=(249, 251, 255), dark=(140, 150, 170)),
    "budget": dict(base=(126, 126, 130), hi=(158, 158, 162), dark=(84, 84, 88)),
    "alum":   dict(base=(198, 200, 204), hi=(226, 228, 232), dark=(140, 142, 148)),
    "iron":   dict(base=(105, 105, 112), hi=(150, 150, 158), dark=(70, 70, 76)),
}
BRASS = (186, 150, 60)
BRASS_HI = (224, 190, 100)
WOOD = dict(base=(124, 86, 52), hi=(152, 110, 68), dark=(92, 60, 34))
WOOD_DARK = dict(base=(104, 74, 48), hi=(128, 94, 60), dark=(76, 52, 32))
GRIPS = {  # grade -> (wrap color, band color) ; None = bare wood
    "lc": None,
    "mc": ((110, 72, 44), (84, 52, 30)),
    "hc": ((74, 54, 38), (52, 36, 24)),
    "ch": ((52, 52, 58), (34, 34, 40)),
    "qt": ((84, 60, 40), BRASS),
    "std": ((110, 72, 44), (84, 52, 30)),
}


def C(rgb):
    return (rgb[0], rgb[1], rgb[2], 255)


# ---------------------------------------------------------------- frame
class Frame:
    """Local frame: t along B->T (0 at grip/butt, 1 at head/tip),
    w across in pixels (perp points down-right on screen)."""

    def __init__(self, B=(4.5, 27.5), T=(27.5, 4.5)):
        self.B = B
        self.T = T
        dx, dy = T[0] - B[0], T[1] - B[1]
        self.len = math.hypot(dx, dy)
        ux, uy = dx / self.len, dy / self.len
        self.u = (ux, uy)
        self.v = (-uy, ux)  # rotate u by +90deg in screen coords (y down)

    def P(self, t, w):
        s = t * self.len
        return (self.B[0] + self.u[0] * s + self.v[0] * w,
                self.B[1] + self.u[1] * s + self.v[1] * w)

    def poly(self, d, pts, fill, outline=None):
        d.polygon([self.P(t, w) for t, w in pts],
                  fill=C(fill), outline=C(outline) if outline else None)

    def line(self, d, t0, w0, t1, w1, fill, width=1):
        d.line([self.P(t0, w0), self.P(t1, w1)], fill=C(fill), width=width)

    def seg(self, d, pts, fill, width=1):
        d.line([self.P(t, w) for t, w in pts], fill=C(fill), width=width)

    def disc(self, d, t, w, r, fill):
        x, y = self.P(t, w)
        d.ellipse([x - r, y - r, x + r, y + r], fill=C(fill))


# ---------------------------------------------------------------- helpers
def haft(d, f, t1, hw, wood=WOOD, t0=0.02):
    f.poly(d, [(t0, -hw), (t1, -hw), (t1, hw), (t0, hw)], wood["base"])
    f.line(d, t0, -hw, t1, -hw, wood["dark"], 1)
    f.line(d, t0, hw, t1, hw, wood["dark"], 1)
    f.line(d, t0 + 0.03, -hw * 0.35, t1 - 0.03, -hw * 0.35, wood["hi"], 1)


def grip(d, f, grade, t1=0.30, hw=1.3):
    g = GRIPS.get(grade)
    if g is None:
        return
    wrap, band = g
    f.poly(d, [(0.02, -hw), (t1, -hw), (t1, hw), (0.02, hw)], wrap)
    t = 0.045
    while t < t1 - 0.01:
        f.line(d, t, -hw, t + 0.018, hw, band, 1)
        t += 0.038
    if grade == "qt":
        f.poly(d, [(t1, -hw - 0.15), (t1 + 0.025, -hw - 0.15),
                   (t1 + 0.025, hw + 0.15), (t1, hw + 0.15)], BRASS)


# ---------------------------------------------------------------- battle axe
def draw_battleaxe(d, f, rng, grade="mc", quality="good"):
    pal = STEEL["alum"] if quality == "fake" else (
        STEEL["budget"] if quality == "inferior" else STEEL[grade])
    haft(d, f, 0.86, 1.3)
    grip(d, f, grade if quality == "good" else "std")
    askew = 0.9 if quality == "inferior" else 0.0  # head droops toward +w
    # poll (back of head)
    f.poly(d, [(0.795, 1.0), (0.80, 2.7 + askew * 0.3), (0.925, 2.7 + askew * 0.3),
               (0.94, 1.0)], pal["dark"])
    # blade: eye at front, flaring to a curved cutting edge on -w side
    blade = [(0.79, -1.2), (0.80, -4.6 + askew), (0.845, -5.9 + askew),
             (0.905, -6.3 + askew), (0.95, -5.2 + askew), (0.945, -1.2)]
    f.poly(d, blade, pal["base"], outline=pal["dark"])
    # cheek shading near the eye
    f.poly(d, [(0.80, -1.4), (0.815, -4.2 + askew), (0.87, -4.4 + askew),
               (0.885, -1.4)], pal["dark"])
    f.poly(d, [(0.83, -1.4), (0.845, -4.0 + askew), (0.875, -4.05 + askew),
               (0.885, -1.4)], pal["base"])
    if quality == "fake":
        # blunt thick edge, no bright line
        f.seg(d, [(0.845, -5.9 + askew), (0.905, -6.3 + askew),
                  (0.95, -5.2 + askew)], pal["dark"], 2)
    else:
        f.seg(d, [(0.845, -5.9 + askew), (0.905, -6.3 + askew),
                  (0.95, -5.2 + askew)], pal["hi"], 1)
    if quality == "inferior":
        # socket gap + a nick on the edge
        f.line(d, 0.775, -1.6, 0.775, 1.6, (40, 36, 32), 1)
        f.disc(d, 0.925, -5.9 + askew, 0.85, (34, 34, 38))
    # eye slot over the haft
    f.poly(d, [(0.845, -1.15), (0.885, -1.15), (0.885, 1.15), (0.845, 1.15)],
           WOOD["dark"])
    if grade == "qt" and quality == "good":
        f.disc(d, 0.865, 0, 0.8, BRASS)


# ---------------------------------------------------------------- war hammer
def draw_warhammer(d, f, rng, grade="mc", quality="good", crowbar=False):
    pal = STEEL["alum"] if quality == "fake" else STEEL[grade]
    tc = 0.90 if crowbar else 0.87
    haft(d, f, tc + 0.02, 1.1 if crowbar else 1.25)
    grip(d, f, grade if quality == "good" else "std", t1=0.28,
         hw=1.1 if crowbar else 1.25)
    s = 0.82 if quality == "fake" else 1.0  # fake head is smaller
    droop = 0.7 if quality == "inferior" else 0.0
    t0, t1 = tc - 0.065 * s - 0.01, tc + 0.065 * s + 0.01
    # hammer face block (-w side), flat striking face
    f.poly(d, [(t0, -1.0), (t0, -4.3 * s + droop), (t1, -4.3 * s),
               (t1, -1.0)], pal["base"], outline=pal["dark"])
    f.poly(d, [(t0, -4.3 * s + droop), (t0 + 0.018, -4.3 * s + droop),
               (t0 + 0.018, -1.2), (t0, -1.2)], pal["hi"])
    # central socket block
    f.poly(d, [(t0, -1.2), (t1, -1.2), (t1, 1.2), (t0, 1.2)], pal["dark"])
    f.poly(d, [(t0 + 0.015, -1.0), (t1 - 0.015, -1.0), (t1 - 0.015, 1.0),
               (t0 + 0.015, 1.0)], pal["base"])
    if crowbar:
        # forked crowbar flat (+w side): flat bar with a V notch
        f.poly(d, [(t0 + 0.01, 1.0), (t1 - 0.01, 1.0), (t1 - 0.005, 4.6),
                   (tc + 0.028, 6.3), (tc + 0.008, 4.9), (tc - 0.012, 6.3),
                   (t0 + 0.005, 4.6)], pal["base"], outline=pal["dark"])
        f.line(d, t0 + 0.02, 2.0, t1 - 0.03, 4.4, pal["hi"], 1)
    else:
        # back spike (+w side), tapering; inferior bends and dulls it
        bend = -0.035 if quality == "inferior" else 0.0
        spike = [(t0 + 0.008, 1.0), (t1 - 0.008, 1.0),
                 (tc + 0.012, 4.0), (tc + bend, 6.6 * s)]
        f.poly(d, spike, pal["base"], outline=pal["dark"])
        if quality != "inferior":
            f.seg(d, [(t1 - 0.014, 1.2), (tc + 0.006, 4.0), (tc + bend, 6.4 * s)],
                  pal["hi"], 1)


# ---------------------------------------------------------------- lucerne
def draw_lucerne_fake(d, f, rng):
    pal = STEEL["alum"]
    haft(d, f, 0.90, 1.15)
    tc = 0.88
    # hammer face (-w)
    f.poly(d, [(tc - 0.055, -1.0), (tc - 0.055, -4.1), (tc + 0.055, -4.1),
               (tc + 0.055, -1.0)], pal["base"], outline=pal["dark"])
    # socket
    f.poly(d, [(tc - 0.06, -1.2), (tc + 0.06, -1.2), (tc + 0.06, 1.2),
               (tc - 0.06, 1.2)], pal["dark"])
    # top spike -- short with a rounded blunt tip on the fake
    f.poly(d, [(0.845, -1.05), (0.915, -1.05), (0.935, 0), (0.915, 1.05),
               (0.845, 1.05)], pal["base"], outline=pal["dark"])
    f.disc(d, 1.045, 0, 1.15, pal["base"])  # rounded blunt tip
    # hook / bec (+w side), thick and blunt
    f.poly(d, [(tc + 0.03, 0.9), (tc + 0.085, 1.4), (tc + 0.10, 3.6),
               (tc + 0.055, 5.4), (tc - 0.005, 5.0), (tc + 0.015, 3.4),
               (tc - 0.02, 1.3)], pal["base"], outline=pal["dark"])


# ---------------------------------------------------------------- macuahuitl
BLADE_MATS = {
    "razor": dict(base=(150, 150, 155), hi=(196, 196, 200), dark=(104, 104, 110)),
    "stone": dict(base=(128, 124, 118), hi=(168, 163, 155), dark=(88, 84, 79)),
    "glass": dict(base=(168, 214, 178), hi=(224, 246, 228), dark=(96, 150, 112)),
    "teeth": dict(base=(232, 224, 200), hi=(248, 243, 226), dark=(178, 168, 142)),
}


def draw_macuahuitl(d, f, rng, blade="mc"):
    pal = BLADE_MATS.get(blade) or STEEL[blade]
    # grip
    f.poly(d, [(0.0, -1.25), (0.16, -1.25), (0.16, 1.25), (0.0, 1.25)],
           (84, 56, 34))
    t = 0.02
    while t < 0.15:
        f.line(d, t, -1.25, t + 0.014, 1.25, (58, 36, 20), 1)
        t += 0.032
    # paddle body, slightly tapering to a rounded tip
    body = [(0.14, -2.75), (0.55, -2.95), (0.93, -2.45), (0.985, -1.1),
            (0.985, 1.1), (0.93, 2.45), (0.55, 2.95), (0.14, 2.75)]
    f.poly(d, body, WOOD["base"], outline=WOOD["dark"])
    f.seg(d, [(0.2, -1.5), (0.6, -1.7), (0.9, -1.2)], WOOD["dark"], 1)
    f.seg(d, [(0.2, 1.6), (0.6, 1.75), (0.9, 1.25)], WOOD["dark"], 1)
    f.seg(d, [(0.2, 0), (0.6, 0.15), (0.9, 0)], WOOD["hi"], 1)
    # blades along both edges
    n = 8
    for i in range(n):
        tb = 0.235 + i * 0.089
        for sgn in (-1, 1):
            w0 = 2.62 * sgn
            if blade == "razor":
                ln = rng.uniform(1.1, 1.9)
                pts = [(tb - 0.030, w0), (tb + 0.030, w0),
                       (tb + rng.uniform(-0.012, 0.012), w0 + sgn * ln)]
                f.poly(d, pts, pal["base"], outline=pal["dark"])
            elif blade == "stone":
                ln = rng.uniform(1.2, 1.7)
                pts = [(tb - 0.034, w0), (tb + 0.034, w0),
                       (tb + 0.014, w0 + sgn * ln), (tb - 0.02, w0 + sgn * ln * 0.8)]
                f.poly(d, pts, pal["base"], outline=pal["dark"])
            elif blade == "teeth":
                pts = [(tb - 0.026, w0), (tb + 0.026, w0),
                       (tb + 0.006, w0 + sgn * 1.75)]
                f.poly(d, pts, pal["base"], outline=pal["dark"])
            elif blade == "glass":
                pts = [(tb - 0.028, w0), (tb + 0.028, w0),
                       (tb, w0 + sgn * 1.8)]
                f.poly(d, pts, pal["base"], outline=pal["dark"])
                f.disc(d, tb, w0 + sgn * 1.15, 0.5, pal["hi"])
            else:  # steel grades: neat rectangular blades with bright edge
                pts = [(tb - 0.031, w0), (tb + 0.031, w0),
                       (tb + 0.026, w0 + sgn * 1.55), (tb - 0.026, w0 + sgn * 1.55)]
                f.poly(d, pts, pal["base"], outline=pal["dark"])
                f.line(d, tb - 0.026, w0 + sgn * 1.55, tb + 0.026,
                       w0 + sgn * 1.55, pal["hi"], 1)
    if blade == "qt":
        f.disc(d, 0.09, 0, 0.85, BRASS)


# ---------------------------------------------------------------- bats
def bat_profile(d, f, hw_fn, fill, outline=None):
    ts = [i / 24.0 for i in range(25)]
    top = [(t, -hw_fn(t)) for t in ts]
    bot = [(t, hw_fn(t)) for t in reversed(ts)]
    f.poly(d, top + bot, fill, outline=outline)


def draw_bat_bolt(d, f, rng):
    pal = STEEL["alum"]
    def hw(t):
        if t < 0.03: return 1.7
        if t < 0.34: return 1.05
        if t < 0.62: return 1.05 + (t - 0.34) * 5.4
        if t < 0.93: return 3.05
        return 3.05 - (t - 0.93) * 16.0
    bat_profile(d, f, hw, pal["base"], pal["dark"])
    # plastic handle
    def hw_h(t):
        return 1.75 if t < 0.03 else 1.1
    ts = [i / 24.0 for i in range(9)]
    pts = [(t, -hw_h(t)) for t in ts] + [(t, hw_h(t)) for t in reversed(ts)]
    f.poly(d, pts, (58, 58, 62))
    f.seg(d, [(0.36, -2.2), (0.7, -2.9), (0.92, -2.4)], pal["hi"], 1)
    # bolts threaded through the barrel
    for tb, wb in [(0.66, -2.3), (0.66, 2.3), (0.78, -2.55), (0.78, 2.55),
                   (0.90, -2.2), (0.90, 2.2), (0.78, 0)]:
        f.disc(d, tb, wb, 1.05, STEEL["iron"]["dark"])
        f.disc(d, tb, wb, 0.5, STEEL["iron"]["hi"])


def draw_goedendag(d, f, rng):
    def hw(t):
        if t < 0.03: return 1.7
        if t < 0.34: return 1.05
        if t < 0.60: return 1.05 + (t - 0.34) * 5.2
        if t < 0.86: return 2.95
        return 2.95 - (t - 0.86) * 8.0
    bat_profile(d, f, hw, WOOD["base"], WOOD["dark"])
    f.seg(d, [(0.38, -1.8), (0.68, -2.6), (0.85, -2.2)], WOOD["hi"], 1)
    # iron cap over the end
    ts = [0.845 + i * 0.008 for i in range(8)]
    pts = [(t, -hw(t) - 0.12) for t in ts] + [(t, hw(t) + 0.12) for t in reversed(ts)]
    f.poly(d, pts, STEEL["iron"]["base"], STEEL["iron"]["dark"])
    # spike straight out the top
    f.poly(d, [(0.885, -0.85), (0.885, 0.85), (1.09, 0)], STEEL["iron"]["base"],
           outline=STEEL["iron"]["dark"])
    f.line(d, 0.90, -0.3, 1.06, -0.05, STEEL["iron"]["hi"], 1)


# ---------------------------------------------------------------- sledge crude
def draw_sledge_crude(d, f, rng):
    haft(d, f, 0.80, 1.4)
    f.disc(d, 0.3, 0.4, 0.7, WOOD["dark"])
    f.disc(d, 0.55, -0.5, 0.6, WOOD["dark"])
    weld = (178, 178, 184)
    # three mismatched welded chunks
    f.poly(d, [(0.74, -3.5), (0.90, -3.5), (0.90, 0.2), (0.74, 0.2)],
           STEEL["mc"]["dark"], outline=(52, 52, 56))
    f.poly(d, [(0.83, -0.5), (0.99, -0.5), (0.99, 3.1), (0.83, 3.1)],
           STEEL["budget"]["base"], outline=(52, 52, 56))
    f.poly(d, [(0.71, 0.5), (0.86, 0.5), (0.86, 3.5), (0.71, 3.5)],
           STEEL["iron"]["base"], outline=(52, 52, 56))
    # weld seams along the joins
    f.seg(d, [(0.90, -3.4), (0.885, -1.6), (0.90, 0.1)], weld, 1)
    f.seg(d, [(0.845, 0.55), (0.86, 1.9), (0.845, 3.4)], weld, 1)
    f.seg(d, [(0.75, -3.3), (0.88, -3.3)], STEEL["mc"]["hi"], 1)


# ---------------------------------------------------------------- canes
def draw_cane(d, f, rng, kind, on=False):
    if kind == "makeshift":
        f.poly(d, [(0.02, -1.2), (0.88, -1.2), (0.88, 1.2), (0.02, 1.2)],
               WOOD_DARK["base"], outline=WOOD_DARK["dark"])
        f.line(d, 0.05, -0.4, 0.85, -0.4, WOOD_DARK["hi"], 1)
        # crook handle hooking toward +w
        f.seg(d, [(0.88, 0), (0.945, 0.5), (0.975, 1.7), (0.955, 3.1),
                  (0.90, 3.8)], WOOD_DARK["base"], 3)
        f.seg(d, [(0.88, 0), (0.945, 0.5), (0.975, 1.7), (0.955, 3.1)],
              WOOD_DARK["dark"], 1)
        f.poly(d, [(0.0, -1.3), (0.05, -1.3), (0.05, 1.3), (0.0, 1.3)],
               (46, 46, 50))
        return
    if kind == "sinister":
        red = (178, 32, 38)
        f.poly(d, [(0.02, -1.15), (0.90, -1.15), (0.90, 1.15), (0.02, 1.15)],
               red, outline=(110, 18, 22))
        f.line(d, 0.05, -0.45, 0.88, -0.45, (232, 96, 96), 1)
        f.poly(d, [(0.0, -1.25), (0.05, -1.25), (0.05, 1.25), (0.0, 1.25)],
               (46, 46, 50))
        # orb
        f.disc(d, 0.985, 0, 3.5, (150, 22, 28))
        f.disc(d, 0.975, -0.4, 3.0, (198, 34, 40))
        f.disc(d, 0.955, -1.1, 1.25, (255, 158, 158))
        # claws gripping the orb from the handle side and top
        claw = STEEL["iron"]["dark"]
        for w0, w1 in [(-3.3, -1.4), (3.3, 1.4)]:
            f.poly(d, [(0.90, w0), (0.965, w0 * 0.82), (1.005, w1),
                       (0.945, w1 * 1.5), (0.895, w1 * 1.35)], claw)
        f.poly(d, [(0.90, -0.7), (0.97, -0.5), (1.03, -1.9), (0.985, -2.6),
                   (0.925, -1.2)], claw)
        return
    # wizard canes
    cheap = kind == "cheap"
    if cheap:
        shaft = (52, 84, 178)
        f.poly(d, [(0.02, -1.1), (0.90, -1.1), (0.90, 1.1), (0.02, 1.1)],
               shaft, outline=(30, 50, 110))
        f.line(d, 0.05, -0.4, 0.88, -0.4, (120, 156, 226), 1)
    else:
        f.poly(d, [(0.02, -1.2), (0.90, -1.2), (0.90, 1.2), (0.02, 1.2)],
               WOOD["base"], outline=WOOD["dark"])
        for tb in (0.32, 0.60):
            f.poly(d, [(tb, -1.3), (tb + 0.028, -1.3), (tb + 0.028, 1.3),
                       (tb, 1.3)], STEEL["alum"]["base"])
    f.poly(d, [(0.0, -1.25), (0.05, -1.25), (0.05, 1.25), (0.0, 1.25)],
           (46, 46, 50))
    r = 2.8 if cheap else 3.3
    tc = 0.975
    if on:
        # opaque halo rings fake the glow (opacity rule)
        f.disc(d, tc, 0, r + (1.9 if cheap else 2.6), (198, 214, 240))
        f.disc(d, tc, 0, r + (1.0 if cheap else 1.5), (228, 238, 252))
    # prong cup under the orb
    for w0 in (-1.9, 0.0, 1.9):
        f.poly(d, [(0.885, w0 - 0.45), (0.93, w0 - 0.3), (0.945, w0 + 0.55),
                   (0.885, w0 + 0.45)], STEEL["iron"]["base"])
    f.disc(d, tc, 0, r, (172, 178, 186) if not on else (244, 248, 252))
    f.disc(d, tc - 0.008, -0.3, r * 0.82, (208, 212, 218) if not on else (252, 253, 255))
    f.disc(d, tc - 0.02, -0.9, r * 0.34, (238, 240, 244))


# ---------------------------------------------------------------- crutches
def draw_crutches(d, f, rng, wooden=False):
    rail = WOOD["base"] if wooden else (168, 170, 176)
    rail_dark = WOOD["dark"] if wooden else (110, 112, 118)
    pad_c = WOOD_DARK["base"] if wooden else (70, 70, 74)
    for c in (-4.8, 4.8):
        # top pad
        f.line(d, 0.965, c - 2.3, 0.965, c + 2.3, pad_c, 3)
        # two rails converging to a single post
        f.line(d, 0.94, c - 2.0, 0.32, c - 0.35, rail, 2)
        f.line(d, 0.94, c + 2.0, 0.32, c + 0.35, rail, 2)
        f.line(d, 0.94, c - 2.0, 0.32, c - 0.35, rail_dark, 1)
        # handgrip
        f.line(d, 0.55, c - 1.75, 0.55, c + 1.75, pad_c, 2)
        # post + tip
        f.line(d, 0.32, c, 0.045, c, rail, 2)
        if not wooden:
            f.line(d, 0.055, c, 0.0, c, (40, 40, 44), 3)


# ---------------------------------------------------------------- singles
def draw_hockey(d, f, rng):
    shaft = (216, 211, 201)
    f.poly(d, [(0.13, -1.15), (0.995, -1.15), (0.995, 1.15), (0.13, 1.15)],
           shaft, outline=(168, 162, 150))
    f.line(d, 0.2, -0.4, 0.95, -0.4, (188, 182, 168), 1)
    f.poly(d, [(0.90, -1.25), (0.995, -1.25), (0.995, 1.25), (0.90, 1.25)],
           (52, 84, 178))  # taped top grip, blue
    # blade: flat, black-taped, extending toward +w at the butt end
    blade = [(0.155, -1.0), (0.16, 1.05), (0.075, 1.35), (0.015, 5.4),
             (0.045, 6.15), (0.115, 2.4), (0.135, 0.5)]
    f.poly(d, blade, (36, 36, 40), outline=(16, 16, 18))
    for tb, w0, w1 in [(0.115, 1.8, 2.6), (0.075, 3.2, 4.0), (0.045, 4.6, 5.3)]:
        f.line(d, tb, w0, tb + 0.035, w1, (74, 74, 80), 1)


def draw_sap(img, rng):
    d = ImageDraw.Draw(img)
    # flat leather pouch, slightly diagonal, with a wrist loop
    d.ellipse([6, 10, 26, 23], fill=C((76, 74, 78)), outline=C((44, 42, 46)))
    d.ellipse([8, 12, 24, 21], outline=C((140, 136, 138)))
    d.arc([6, 10, 26, 23], start=200, end=320, fill=C((104, 100, 104)), width=1)
    d.line([11, 16, 21, 16], fill=C((58, 56, 60)), width=1)
    # wrist strap loop
    d.arc([20, 3, 30, 13], start=270, end=120, fill=C((58, 50, 44)), width=2)
    d.arc([20, 3, 30, 13], start=270, end=120, fill=C((96, 84, 70)), width=1)


def draw_shank(d, f, rng):
    blue = (140, 190, 225)
    hi = (203, 232, 250)
    dark = (86, 138, 178)
    f.poly(d, [(0.02, -1.35), (0.72, -1.35), (0.72, 1.35), (0.02, 1.35)],
           blue, outline=dark)
    for tb in (0.10, 0.17, 0.24, 0.31):
        f.line(d, tb, -1.35, tb, 1.35, dark, 1)
    f.line(d, 0.04, -0.5, 0.70, -0.5, hi, 1)
    # neck, then the ground point (faceted: bright upper face)
    f.poly(d, [(0.72, -0.95), (0.83, -0.8), (0.83, 0.8), (0.72, 0.95)], blue)
    f.poly(d, [(0.83, -0.8), (1.0, 0), (0.83, 0)], hi, outline=dark)
    f.poly(d, [(0.83, 0), (1.0, 0), (0.83, 0.8)], blue, outline=dark)


# ---------------------------------------------------------------- dispatch
def render(eid):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    rng = random.Random(zlib.crc32(eid.encode()))
    f = Frame()
    if eid.endswith("_aztec_sword") or eid in ("aztec_sword_scrap",
                                              "aztec_sword_stone",
                                              "glass_macuahuitl",
                                              "teeth_macuahuitl"):
        blade = {"aztec_sword_scrap": "razor", "aztec_sword_stone": "stone",
                 "glass_macuahuitl": "glass", "teeth_macuahuitl": "teeth"}
        if eid in blade:
            draw_macuahuitl(d, f, rng, blade[eid])
        else:
            draw_macuahuitl(d, f, rng, eid.split("_")[0])
    elif "battleaxe" in eid:
        if eid.endswith("_fake"):
            draw_battleaxe(d, f, rng, quality="fake")
        elif eid.endswith("_inferior"):
            draw_battleaxe(d, f, rng, quality="inferior")
        elif eid == "battleaxe":
            draw_battleaxe(d, f, rng, grade="mc")
        else:
            draw_battleaxe(d, f, rng, grade=eid.split("_")[0])
    elif "warhammer" in eid:
        if eid == "crowbar_warhammer":
            draw_warhammer(d, f, rng, grade="mc", crowbar=True)
        elif eid.endswith("_fake"):
            draw_warhammer(d, f, rng, quality="fake")
        elif eid.endswith("_inferior"):
            draw_warhammer(d, f, rng, quality="inferior")
        else:
            draw_warhammer(d, f, rng, grade=eid.split("_")[0])
    elif eid == "lucern_hammerfake":
        draw_lucerne_fake(d, f, rng)
    elif eid == "bat_metal_bolt":
        draw_bat_bolt(d, f, rng)
    elif eid == "goedendag_bat":
        draw_goedendag(d, f, rng)
    elif eid == "hammer_sledge_crude":
        draw_sledge_crude(d, f, rng)
    elif eid == "makeshift_cane":
        draw_cane(d, f, rng, "makeshift")
    elif eid == "sinister_cane":
        draw_cane(d, f, rng, "sinister")
    elif eid == "wizard_cane":
        draw_cane(d, f, rng, "wizard", on=False)
    elif eid == "wizard_cane_on":
        draw_cane(d, f, rng, "wizard", on=True)
    elif eid == "wizard_cane_cheap":
        draw_cane(d, f, rng, "cheap", on=False)
    elif eid == "wizard_cane_cheap_on":
        draw_cane(d, f, rng, "cheap", on=True)
    elif eid in ("crutches", "makeshift_crutches"):
        vf = Frame(B=(14.5, 29.5), T=(18.0, 2.5))
        draw_crutches(d, vf, rng, wooden=(eid == "makeshift_crutches"))
    elif eid == "hockey_stick":
        draw_hockey(d, f, rng)
    elif eid == "makeshift_sap":
        draw_sap(img, rng)
    elif eid == "sharp_toothbrush":
        draw_shank(d, f, rng)
    else:
        raise ValueError("no renderer for " + eid)
    alphas = set(img.getchannel("A").getdata())
    assert alphas <= {0, 255}, (eid, sorted(alphas)[:8])
    return img


def main():
    for eid in IDS:
        img = render(eid)
        folder = os.path.join(ITEMS_DIR, eid)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, eid + ".png"))
        with open(os.path.join(folder, eid + ".json"), "w") as fh:
            json.dump([{"id": eid, "fg": eid, "bg": ""}], fh, indent=2)
            fh.write("\n")
        with open(os.path.join(folder, "credits.txt"), "w") as fh:
            fh.write(CREDITS)
    # contact-sheet preview: 8 cols x 5 rows, 2x sprites on dark cells
    cols, rows, cell = 8, 5, 66
    prev = Image.new("RGBA", (cols * cell, rows * cell), (34, 34, 40, 255))
    for i, eid in enumerate(IDS):
        img = render(eid).resize((64, 64), Image.NEAREST)
        x = (i % cols) * cell + 1
        y = (i // cols) * cell + 1
        prev.alpha_composite(img, (x, y))
    prev.save(os.path.join(BASE, "preview_item38.png"))
    print(f"wrote {len(IDS)} sprites + preview_item38.png {prev.size}")


if __name__ == "__main__":
    main()
