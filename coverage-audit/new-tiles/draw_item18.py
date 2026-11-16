#!/usr/bin/env python3
"""Batch ITEM18 (Phase 2g.4 scope extension): swords and
blades, first set.

52 ids from data/json/items/melee/swords_and_blades.json
(authoritative list embedded below; source list:
~/workspace/item-batches/18.txt). Copy-from chains were
resolved against the data before drawing; the steel-grade
families (broadsword / estoc / khopesh / kukri / wakizashi)
share one base entry each, with the grade carried by the
prefix: lc_ mild steel (dull, "lacking a proper grip" --
drawn as a bare tang), mc_ medium steel (leather grip),
hc_ high steel (bright, sturdy grip), ch_ hardened steel
(dark blue steel, molded grip), qt_ tempered steel
(masterpiece: brightest blade, brass fittings). The plain
broadsword / kukri / mc_* entries are the medium grade.
_fake ids are dull cheap replicas (blunt, no bright edge);
_inferior / kirpan_cheap are poorly made (nicks, wobble,
rough finish). The khopesh also exists in bronze (khopesh
line, yellow in the data) and as steel via mc_khopesh --
iron_khopesh resolves to the medium-steel entry in the
data, drawn a touch rougher. The dao is bronze with the
cupped disc guard of its description.

Singles: Type 30 sword bayonet (arisaka_bayonet, with
muzzle ring), Enfield spike bayonet (cruciform spike on a
socket), butterfly sword (wide blade, D knuckle guard),
baselard (iron, H-shaped hilt), kris (wavy blade), kirpan
(ceremonial curved dagger), karambit (claw blade, finger
ring), bone shiv (jagged broken bone), glass shiv (shard
with a cloth wrap), switchblade (blade out of a plastic
handle), punch dagger (T palm grip), bagh nakha (tiger
claws: bar with four curved claws and finger rings),
razorbar katar (five rebar spikes strapped to a forearm
brace).

Renderer: each bladed weapon is drawn in a local frame
(u along the weapon, v across it) rotated 45 degrees --
hilt low-left, tip high-right -- with a per-kind blade
profile (centerline curve + half-width along u) and
per-grade palettes. Items are objects on transparency,
native 32x32, flat UltiCa style, bodies fully opaque
(alpha in {0,255}). PIL only, per-sprite seeded
random.Random (crc32 of id). Writes ONLY its own 52
folders + preview_item18.png.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the "
           "tileset.\n")

IDS = [
    "arisaka_bayonet", "bagh_nakha", "bone_knife", "broadsword",
    "broadsword_fake", "broadsword_inferior", "butterfly_swords",
    "ch_broadsword", "ch_estoc", "ch_khopesh", "ch_kukri",
    "ch_wakizashi", "dao", "enfield_bayonet", "estoc", "estoc_fake",
    "estoc_inferior", "glass_shiv", "hc_broadsword", "hc_estoc",
    "hc_khopesh", "hc_kukri", "hc_wakizashi", "iron_khopesh",
    "iron_khopesh_fake", "iron_khopesh_inferior", "karambit",
    "khopesh", "khopesh_fake", "khopesh_inferior", "kirpan",
    "kirpan_cheap", "knife_baselard", "knuckle_katar", "kris",
    "kris_fake", "kukri", "lc_broadsword", "lc_estoc", "lc_khopesh",
    "lc_kukri", "lc_wakizashi", "mc_estoc", "mc_khopesh",
    "mc_wakizashi", "punch_dagger", "qt_broadsword", "qt_estoc",
    "qt_khopesh", "qt_kukri", "qt_wakizashi", "switchblade",
]

S = math.sqrt(0.5)
DIR = (S, -S)      # along the weapon, hilt -> tip (up-right)
PERP = (S, S)      # across the weapon (+v = edge side, down-right)
CENTER = (15.5, 15.5)


def clamp(x):
    return max(0, min(255, int(round(x))))


def lerp(a, b, t):
    return tuple(clamp(a[i] + (b[i] - a[i]) * t) for i in range(3))


def shade(col, f):
    return tuple(clamp(c * f) for c in col)


def noisy(col, n):
    return tuple(clamp(c + n) for c in col)


# ---------------------------------------------------------------- palettes
# base / highlight / spine(dark back) + finish roughness
PALS = {
    "lc":   dict(base=(148, 150, 157), hi=(181, 183, 189),
                 spine=(119, 121, 128), rough=13),
    "mc":   dict(base=(176, 180, 188), hi=(215, 220, 228),
                 spine=(145, 149, 159), rough=7),
    "hc":   dict(base=(197, 202, 212), hi=(239, 243, 251),
                 spine=(165, 170, 182), rough=4),
    "ch":   dict(base=(129, 141, 163), hi=(190, 202, 224),
                 spine=(103, 115, 140), rough=5),
    "qt":   dict(base=(204, 212, 227), hi=(249, 251, 255),
                 spine=(171, 181, 201), rough=2),
    "fake": dict(base=(138, 138, 141), hi=(161, 161, 163),
                 spine=(123, 123, 126), rough=10),
    "inferior": dict(base=(161, 164, 171), hi=(193, 197, 205),
                     spine=(131, 134, 142), rough=12),
    "iron": dict(base=(155, 157, 163), hi=(197, 199, 205),
                 spine=(127, 129, 136), rough=11),
    "bronze": dict(base=(178, 132, 54), hi=(231, 185, 97),
                   spine=(148, 108, 42), rough=6),
    "bronze_inferior": dict(base=(170, 126, 52), hi=(218, 172, 88),
                            spine=(140, 102, 40), rough=12),
    "bronze_fake": dict(base=(199, 169, 89), hi=(229, 203, 125),
                        spine=(173, 143, 67), rough=8),
    "steel": dict(base=(186, 190, 198), hi=(225, 229, 237),
                  spine=(155, 159, 169), rough=5),
    "darksteel": dict(base=(112, 116, 124), hi=(165, 170, 180),
                      spine=(89, 93, 101), rough=6),
    "bone": dict(base=(228, 222, 204), hi=(245, 241, 229),
                 spine=(201, 194, 172), rough=5),
    "glass": dict(base=(168, 222, 232), hi=(228, 249, 253),
                  spine=(138, 198, 213), rough=4),
}

GRADE_GRIP = {  # sword-family grip style per grade
    "lc": "bare", "mc": "leather", "hc": "sturdy", "ch": "molded",
    "qt": "fine", "fake": "cheap", "inferior": "leather",
    "iron": "leather", "bronze": "leather_dark",
    "bronze_inferior": "leather_dark", "bronze_fake": "cheap",
}
BRASS = (172, 140, 70)
BRASS_D = (138, 110, 52)


def grade_grip(grade, waki=False):
    if waki:
        return {"lc": "wrap_rough", "mc": "wrap", "hc": "wrap",
                "ch": "wrap_dark", "qt": "wrap",
                "fake": "cheap", "inferior": "wrap_rough"}[grade]
    return GRADE_GRIP[grade]


# ------------------------------------------------------------------- specs
# kind, palette, grip style, guard style, grip_len, grip_hw,
# blade_len, half_width, double-edged, fuller
def spec(kind, pal, grip, guard, glen, ghw, blen, hw, dbl=False,
         fuller=False, dark=1.0, blunt=False, nicks=False,
         wobble=False):
    return dict(kind=kind, pal=pal, grip=grip, guard=guard,
                glen=glen, ghw=ghw, blen=blen, hw=hw, dbl=dbl,
                fuller=fuller, dark=dark, blunt=blunt, nicks=nicks,
                wobble=wobble)


SPECS = {}


def build_specs():
    fam = [
        # id -> grade for each family
        ("broadsword", dict(broadsword="mc", lc_broadsword="lc",
                            hc_broadsword="hc", ch_broadsword="ch",
                            qt_broadsword="qt", broadsword_fake="fake",
                            broadsword_inferior="inferior"),
         dict(guard="cross", glen=5.5, ghw=1.15, blen=20.0, hw=1.7,
              dbl=True)),
        ("estoc", dict(estoc="mc", mc_estoc="mc", lc_estoc="lc",
                       hc_estoc="hc", ch_estoc="ch", qt_estoc="qt",
                       estoc_fake="fake", estoc_inferior="inferior"),
         dict(guard="cross", glen=5.5, ghw=1.15, blen=23.0, hw=0.8,
              dbl=True)),
        ("khopesh", dict(mc_khopesh="mc", iron_khopesh="iron",
                         lc_khopesh="lc", hc_khopesh="hc",
                         ch_khopesh="ch", qt_khopesh="qt",
                         iron_khopesh_fake="fake",
                         iron_khopesh_inferior="inferior"),
         dict(guard="small", glen=4.2, ghw=1.1, blen=14.5, hw=1.9,
              fuller=True)),
        ("khopesh", dict(khopesh="bronze", khopesh_fake="bronze_fake",
                         khopesh_inferior="bronze_inferior"),
         dict(guard="small", glen=4.2, ghw=1.1, blen=14.5, hw=1.9,
              fuller=True)),
        ("kukri", dict(kukri="mc", lc_kukri="lc", hc_kukri="hc",
                       ch_kukri="ch", qt_kukri="qt"),
         dict(guard="bolster", glen=4.4, ghw=1.2, blen=12.5, hw=2.2,
              dark=0.74)),
        ("wakizashi", dict(mc_wakizashi="mc", lc_wakizashi="lc",
                           hc_wakizashi="hc", ch_wakizashi="ch",
                           qt_wakizashi="qt"),
         dict(guard="tsuba", glen=5.2, ghw=1.1, blen=16.0, hw=1.25,
              fuller=True)),
    ]
    for kind, grades, kw in fam:
        for gid, grade in grades.items():
            grip = grade_grip(grade, waki=(kind == "wakizashi"))
            if kind == "kukri":
                grip = "wood"
            blunt = grade in ("fake", "bronze_fake")
            nicks = grade in ("inferior", "bronze_inferior")
            wob = nicks
            SPECS[gid] = spec(kind, grade, grip, kw["guard"],
                              kw["glen"], kw["ghw"], kw["blen"],
                              kw["hw"], dbl=kw.get("dbl", False),
                              fuller=kw.get("fuller", False),
                              dark=kw.get("dark", 1.0), blunt=blunt,
                              nicks=nicks, wobble=wob)
    SPECS["dao"] = spec("dao", "bronze", "darkwood", "cup", 5.0, 1.15,
                        19.0, 1.6, fuller=True)
    SPECS["arisaka_bayonet"] = spec("bayonet", "steel", "wood",
                                    "cross_ring", 5.0, 1.15, 15.0,
                                    1.25, fuller=True)
    SPECS["enfield_bayonet"] = spec("spike", "steel", "socket",
                                     "none", 5.2, 1.7, 11.5, 0.62,
                                     dbl=True)
    SPECS["butterfly_swords"] = spec("butterfly", "steel", "leather_dark",
                                     "dgard", 4.4, 1.15, 13.0, 2.0,
                                     fuller=True)
    SPECS["knife_baselard"] = spec("baselard", "iron", "wood_h",
                                   "h_hilt", 4.2, 1.1, 10.5, 1.2,
                                   dbl=True)
    SPECS["kris"] = spec("kris", "darksteel", "carved", "flare",
                         4.2, 1.15, 13.0, 1.3, dbl=True)
    SPECS["kris_fake"] = spec("kris", "fake", "cheap", "flare",
                              4.2, 1.15, 13.0, 1.3, dbl=True, blunt=True)
    SPECS["kirpan"] = spec("kirpan", "darksteel", "leather_dark",
                           "disc", 3.6, 1.05, 7.5, 1.05)
    SPECS["kirpan_cheap"] = spec("kirpan", "inferior", "leather_dark",
                                 "disc", 3.6, 1.05, 7.5, 1.05,
                                 nicks=True, wobble=True)
    SPECS["karambit"] = spec("karambit", "darksteel", "molded",
                             "ring", 4.2, 1.5, 9.5, 1.2)
    SPECS["bone_knife"] = spec("bone", "bone", "bone", "none",
                               5.5, 1.15, 8.5, 1.35)
    SPECS["glass_shiv"] = spec("glass", "glass", "wrap_cloth", "none",
                               4.2, 1.5, 7.5, 1.3)
    SPECS["switchblade"] = spec("switchblade", "steel", "plastic",
                                "bolster_s", 7.5, 1.7, 9.0, 0.95)
    SPECS["punch_dagger"] = spec("punch", "darksteel", "tbar", "none",
                                 3.0, 4.2, 8.5, 1.35, dbl=True)


build_specs()


# ------------------------------------------------------------- blade shape
def blade_profile(kind, u, L):
    """Return (center_offset, half_width) at axial position u."""
    s = u / L if L else 0.0
    if kind == "broadsword":
        c, w = 0.0, 1.0
        if s > 0.72:
            q = (s - 0.72) / 0.28
            w = math.sqrt(max(0.03, 1 - q * q))
        return c, w
    if kind == "estoc":
        c, w = 0.0, 1.0
        if s > 0.82:
            w = max(0.05, 1 - (s - 0.82) / 0.18)
        return c, w
    if kind == "khopesh":
        if s < 0.42:
            return 0.0, 0.78
        q = (s - 0.42) / 0.58
        c = -3.4 * q * q
        w = 0.78 + 0.45 * math.sin(min(q * 1.25, 1.0) * math.pi * 0.5)
        if q > 0.78:
            w *= max(0.25, 1 - (q - 0.78) / 0.22 * 0.75)
        return c, w
    if kind == "kukri":
        c = 2.1 * s * s
        w = 0.60 + 0.62 * math.sin(min(s / 0.68, 1.0) * math.pi * 0.5)
        if s > 0.80:
            q = (s - 0.80) / 0.20
            w *= max(0.10, 1 - q * q * 0.92)
            c += 0.5 * q
        return c, w
    if kind == "wakizashi":
        c = -1.15 * s * s
        w = 1.0
        if s > 0.70:
            q = (s - 0.70) / 0.30
            w = math.sqrt(max(0.04, 1 - q * q))
        return c, w
    if kind == "dao":
        c = -1.35 * s * s
        w = 1.0 + 0.22 * s
        if s > 0.74:
            q = (s - 0.74) / 0.26
            w *= math.sqrt(max(0.04, 1 - q * q))
        return c, w
    if kind == "kris":
        ph = s * math.pi * 3.1
        c = 0.9 * math.sin(ph)
        w = 1.0 + 0.13 * math.sin(ph + 1.2)
        if s > 0.85:
            w *= max(0.08, 1 - (s - 0.85) / 0.15)
        return c, w
    if kind == "bayonet":
        w = 1.0
        if s > 0.80:
            w = math.sqrt(max(0.04, 1 - ((s - 0.80) / 0.20) ** 2))
        return 0.0, w
    if kind == "spike":
        w = 1.0
        if s > 0.78:
            w = max(0.06, 1 - (s - 0.78) / 0.22)
        return 0.0, w
    if kind == "butterfly":
        w = 1.0
        if s > 0.80:
            q = (s - 0.80) / 0.20
            w = math.sqrt(max(0.05, 1 - q * q))
        return 0.0, w
    if kind == "baselard":
        w = 1.0
        if s > 0.68:
            q = (s - 0.68) / 0.32
            w = math.sqrt(max(0.04, 1 - q * q))
        return 0.0, w
    if kind == "kirpan":
        c = -0.8 * s * s
        w = 1.0
        if s > 0.66:
            q = (s - 0.66) / 0.34
            w = math.sqrt(max(0.04, 1 - q * q))
        return c, w
    if kind == "karambit":
        c = 2.6 * s * s
        w = 1.0
        if s > 0.55:
            w = max(0.05, 1 - (s - 0.55) / 0.45)
        return c, w
    if kind == "bone":
        c = 0.3 * math.sin(s * math.pi)
        w = 1.0 + 0.15 * math.sin(u * 2.3 + 1.7)
        if s > 0.60:
            q = (s - 0.60) / 0.40
            w *= max(0.12, 1 - q * 0.85)
        return c, w
    if kind == "glass":
        w = 1.0 + 0.13 * math.sin(u * 2.9 + 0.6)
        if s > 0.55:
            q = (s - 0.55) / 0.45
            w *= max(0.10, 1 - q * 0.9)
        return 0.0, w
    if kind == "switchblade":
        w = 1.0
        if s > 0.62:
            q = (s - 0.62) / 0.38
            w = math.sqrt(max(0.05, 1 - q * q))
        return 0.0, w
    if kind == "punch":
        return 0.0, max(0.06, 1 - 0.82 * s)
    return 0.0, 1.0


def tip_curve(kind, L):
    return blade_profile(kind, L, L)[0]


# --------------------------------------------------------------- grip look
def grip_color(style, u, v, hw, n):
    """u in [-glen, 0], v across the grip."""
    cyl = 1.0 - 0.20 * abs(v) / max(hw, 0.01)
    if style == "leather":
        base, stripe = (112, 80, 50), (85, 59, 35)
        col = stripe if ((u * 1.25 + v * 0.8) % 2.2) < 0.8 else base
    elif style == "leather_dark":
        base, stripe = (94, 64, 38), (70, 46, 26)
        col = stripe if ((u * 1.25 + v * 0.8) % 2.2) < 0.8 else base
    elif style == "sturdy":
        col = (99, 69, 42)
        if u > -0.8 or u < -4.7:
            col = (72, 48, 28)
        elif ((u * 1.4 + v * 0.7) % 2.4) < 0.6:
            col = (82, 56, 33)
    elif style == "molded":
        col = (47, 47, 53) if (u % 1.1) < 0.45 else (61, 61, 67)
    elif style == "fine":
        base, stripe = (47, 49, 63), (76, 78, 100)
        col = stripe if ((u * 1.5 + v) % 2.0) < 0.55 else base
    elif style == "cheap":
        col = (43, 43, 47)
        if v < -hw * 0.45:
            col = (56, 56, 61)
    elif style == "bare":
        col = (141, 143, 149)
        if abs(v) < 0.35 and (abs(u + 1.8) < 0.5 or abs(u + 4.0) < 0.5):
            col = (70, 70, 76)  # rivets
    elif style == "wood":
        col = (108, 75, 42) if math.sin(u * 2.1 + v * 3.0) > 0.72 \
            else (131, 93, 55)
    elif style == "darkwood":
        col = (84, 58, 33) if math.sin(u * 2.3 + v * 2.6) > 0.72 \
            else (104, 73, 44)
    elif style == "carved":
        col = (70, 48, 30) if ((u * 1.8) % 1.6) < 0.5 else (90, 65, 40)
    elif style in ("wrap", "wrap_dark", "wrap_rough"):
        if style == "wrap_dark":
            base, stripe = (30, 32, 48), (52, 54, 74)
        elif style == "wrap_rough":
            base, stripe = (44, 42, 52), (28, 27, 36)
        else:
            base, stripe = (39, 39, 55), (63, 63, 84)
        col = stripe if ((int(u * 1.35) + int(v * 1.35)) % 2 == 0) else base
    elif style == "wrap_cloth":
        col = (147, 139, 121) if (u % 1.2) < 0.55 else (173, 165, 149)
    elif style == "black":
        col = (47, 47, 53) if (u % 1.0) < 0.4 else (34, 34, 39)
    elif style == "plastic":
        col = (45, 45, 53) if (u % 1.3) < 0.4 else (32, 32, 38)
        if u > -1.0:  # bolster
            col = (150, 152, 158)
    elif style == "bone":
        col = (208, 201, 182)
        if math.sin(u * 3.1 + v * 2.0) > 0.85:
            col = (188, 180, 158)
    elif style == "socket":
        col = (89, 93, 101)
        if abs(u + 1.3) < 0.35 or abs(u + 3.6) < 0.35:
            col = (63, 67, 75)
        if u > -0.5:
            col = (99, 103, 111)
    elif style == "tbar":
        col = (64, 64, 70)
        if abs(v) < 1.1:
            col = (88, 80, 66)
    else:
        col = (100, 80, 55)
    return noisy(shade(col, cyl), n // 2)


# -------------------------------------------------------------- rendering
def render_bladed(item_id, sp, rng):
    pal = dict(PALS[sp["pal"]])
    if sp["dark"] != 1.0:
        for k in ("base", "hi", "spine"):
            pal[k] = shade(pal[k], sp["dark"])
    rough = pal["rough"]
    noise = [[rng.randint(-rough, rough) for _ in range(32)]
             for _ in range(32)]
    jag = [rng.uniform(-0.34, 0.34) for _ in range(48)]
    L = sp["blen"]
    nicks = []
    if sp["nicks"]:
        pop = list(range(3, max(4, int(L) - 1)))
        nicks = sorted(rng.sample(pop, min(2, len(pop))))
    glen, ghw = sp["glen"], sp["ghw"]
    c_tip = tip_curve(sp["kind"], L)
    mid_u = (L - glen - 1.3) / 2.0
    ox = CENTER[0] - DIR[0] * mid_u - PERP[0] * c_tip * 0.5
    oy = CENTER[1] - DIR[1] * mid_u - PERP[1] * c_tip * 0.5
    guard_col = shade(pal["spine"], 0.82)
    if sp["pal"] == "qt":
        guard_col = BRASS
    elif sp["pal"] == "ch":
        guard_col = (72, 79, 95)
    elif sp["pal"] in ("bronze", "bronze_inferior", "bronze_fake"):
        guard_col = shade(pal["base"], 0.85)

    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    px = img.load()
    for y in range(32):
        for x in range(32):
            dx, dy = x - ox, y - oy
            u = dx * DIR[0] + dy * DIR[1]
            v = dx * PERP[0] + dy * PERP[1]
            n = noise[y][x]
            col = None
            # ---- blade
            if 0.0 <= u <= L:
                c, wf = blade_profile(sp["kind"], u, L)
                if sp["wobble"]:
                    c += 0.45 * math.sin(u * 1.25 + 0.7)
                hw = sp["hw"] * wf
                dv = v - c
                if sp["kind"] in ("bone", "glass"):
                    w_edge = hw + jag[int(u * 2) % 48]
                    w_sp = hw * 0.9
                    if -w_sp <= dv <= w_edge:
                        t = (dv + w_sp) / (w_sp + w_edge)
                        col = blade_shade(pal, t, sp, n)
                elif abs(dv) <= hw:
                    t = dv / (2 * hw) + 0.5
                    col = blade_shade(pal, t, sp, n)
                    if nicks and any(abs(u - nk) < 0.55 for nk in nicks) \
                            and t > 0.78:
                        col = (58, 58, 63, 255)
            # ---- guard (over grip, under blade start)
            if col is None:
                col = guard_shade(sp, u, v, guard_col, pal, n)
            # ---- grip / handle
            if col is None and -glen <= u <= 0.1:
                hw_g = ghw
                if sp["grip"] == "bare":
                    hw_g = 0.8
                if sp["grip"] == "bone":
                    # knobby bone end: condyle bumps
                    d1 = math.hypot(u + 5.2, v - 0.2)
                    d2 = math.hypot(u + 4.0, v + 0.4)
                    if d1 < 1.55 or d2 < 1.25:
                        col = noisy(pal["base"], n)
                    elif abs(v) <= hw_g:
                        col = grip_color("bone", u, v, hw_g, n)
                elif abs(v) <= hw_g:
                    col = grip_color(sp["grip"], u, v, hw_g, n)
            # ---- pommel
            if col is None and sp["guard"] not in (
                    "h_hilt", "ring", "none", "dgard") \
                    and sp["kind"] not in ("punch", "spike",
                                           "switchblade", "bone"):
                du, dvv = u + glen + 1.05, v
                if du * du + dvv * dvv <= 1.3 * 1.3:
                    pc = BRASS if sp["pal"] == "qt" else guard_col
                    col = noisy(shade(pc, 0.92), n)
            if col is not None:
                if len(col) == 3:
                    col = col + (255,)
                px[x, y] = col
    draw_specials(img, item_id, sp, (ox, oy), pal, guard_col, rng)
    return img


def blade_shade(pal, t, sp, n):
    base, hi, spine = pal["base"], pal["hi"], pal["spine"]
    hi_b = tuple(clamp(c * 1.10 + 12) for c in hi)
    ridge = 0.5 if sp["dbl"] else 0.40
    if not sp["blunt"] and t > 0.87:
        col = hi_b
    elif not sp["blunt"] and sp["dbl"] and t < 0.13:
        col = hi_b
    elif abs(t - ridge) <= 0.13:
        col = hi
    elif sp["fuller"] and not sp["dbl"] and t < 0.22:
        col = shade(spine, 0.88)
    elif t < ridge:
        col = lerp(spine, base, t / ridge)
    else:
        col = lerp(base, hi, (t - ridge) / (1 - ridge) * 0.55)
    return noisy(col, n) + (255,)


def guard_shade(sp, u, v, guard_col, pal, n):
    g = sp["guard"]
    if g == "cross" or g == "cross_ring":
        gw = 2.7 if sp["kind"] == "broadsword" else 2.35
        if -0.85 <= u <= 0.45 and abs(v) <= gw:
            if abs(v) > gw - 0.6 and (u < -0.55 or u > 0.2):
                return None
            return noisy(guard_col, n) + (255,)
    elif g == "tsuba":
        e = (u / 0.8) ** 2 + (v / 2.7) ** 2
        if e <= 1.0:
            if sp["pal"] == "qt" and e > 0.72:
                return noisy(BRASS, n) + (255,)
            return noisy((74, 74, 81), n) + (255,)
    elif g == "cup":
        e = (u / 0.95) ** 2 + (v / 2.5) ** 2
        if e <= 1.0:
            f = 0.85 + 0.25 * (1 - e)
            return noisy(shade(pal["base"], f), n) + (255,)
    elif g == "small":
        if -0.7 <= u <= 0.4 and abs(v) <= 1.85:
            return noisy(guard_col, n) + (255,)
    elif g == "flare":  # kris ganja: wider on the spine side
        if -0.7 <= u <= 0.4 and -2.35 <= v <= 1.75:
            return noisy(guard_col, n) + (255,)
    elif g == "bolster":
        if -0.6 <= u <= 0.35 and abs(v) <= 1.6:
            return noisy((158, 132, 78), n) + (255,)
    elif g == "disc":
        e = (u / 0.7) ** 2 + (v / 1.9) ** 2
        if e <= 1.0:
            return noisy(guard_col, n) + (255,)
    elif g == "bolster_s":
        if -1.0 <= u <= 0.1 and abs(v) <= 1.5:
            return noisy((150, 152, 158), n) + (255,)
    elif g == "h_hilt":
        # baselard: guard bar + pommel bar (the I-beam hilt)
        if -0.8 <= u <= 0.5 and abs(v) <= 3.0:
            return noisy((88, 88, 94), n) + (255,)
        glen = sp["glen"]
        if -glen - 0.9 <= u <= -glen + 0.4 and abs(v) <= 3.0:
            return noisy((88, 88, 94), n) + (255,)
    elif g == "dgard":
        if -0.7 <= u <= 0.4 and abs(v) <= 2.0:
            return noisy(guard_col, n) + (255,)
    return None


def to_xy(origin, u, v):
    return (origin[0] + u * DIR[0] + v * PERP[0],
            origin[1] + u * DIR[1] + v * PERP[1])


def draw_specials(img, item_id, sp, origin, pal, guard_col, rng):
    d = ImageDraw.Draw(img)
    steel_d = shade(pal["spine"], 0.8) + (255,)
    if sp["guard"] == "dgard":  # butterfly sword D knuckle guard
        pts = []
        glen = sp["glen"]
        for i in range(13):
            t = i / 12.0
            u = 0.2 - (glen + 0.1) * t
            v = 1.0 + 2.5 * math.sin(t * math.pi)
            pts.append(to_xy(origin, u, v))
        d.line(pts, fill=steel_d, width=2)
    if sp["guard"] == "cross_ring":  # arisaka muzzle ring
        cx, cy = to_xy(origin, 0.6, 3.1)
        d.ellipse([cx - 1.7, cy - 1.7, cx + 1.7, cy + 1.7],
                  outline=steel_d, width=1)
        bx, by = to_xy(origin, 0.2, 2.2)
        d.line([bx, by, cx, cy], fill=steel_d, width=1)
    if sp["kind"] == "dao":  # pommel ring
        cx, cy = to_xy(origin, -sp["glen"] - 2.0, 0)
        d.ellipse([cx - 1.6, cy - 1.6, cx + 1.6, cy + 1.6],
                  outline=BRASS_D + (255,), width=1)
        bx, by = to_xy(origin, -sp["glen"] - 0.6, 0)
        d.line([bx, by, cx, cy], fill=BRASS_D + (255,), width=1)
    if sp["guard"] == "ring":  # karambit finger ring + bridge
        ring_col = shade(pal["hi"], 0.92) + (255,)
        cx, cy = to_xy(origin, -sp["glen"] - 2.1, 0)
        d.ellipse([cx - 2.1, cy - 2.1, cx + 2.1, cy + 2.1],
                  outline=ring_col, width=2)
        p1 = to_xy(origin, -sp["glen"] - 0.9, 0)
        d.line([p1, (cx, cy)], fill=ring_col, width=2)
    if item_id == "switchblade":  # release button
        bx, by = to_xy(origin, -3.4, 0)
        d.ellipse([bx - 0.8, by - 0.8, bx + 0.8, by + 0.8],
                  fill=(168, 170, 176, 255))


def render_bagh_nakha(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    pal = PALS["darksteel"]
    rough = pal["rough"]
    noise = [[rng.randint(-rough, rough) for _ in range(32)]
             for _ in range(32)]
    base = pal["base"] + (255,)
    dark = shade(pal["spine"], 0.8) + (255,)
    hi = pal["hi"] + (255,)
    # palm bar
    d.rectangle([4, 9, 27, 12], fill=base)
    d.line([4, 9, 27, 9], fill=hi, width=1)
    d.line([4, 12, 27, 12], fill=dark, width=1)
    # finger rings hanging at both ends
    for cx in (7, 24):
        d.rectangle([cx - 1, 12, cx + 1, 14], fill=dark)
        d.ellipse([cx - 3.0, 13.5, cx + 3.0, 19.5], outline=dark, width=2)
        d.arc([cx - 3.0, 13.5, cx + 3.0, 19.5], start=180, end=300,
              fill=hi, width=1)
    # four curved claws, long enough to read as claws
    for cx in (10.5, 14.5, 18.5, 22.5):
        claw = [(cx - 1.6, 12), (cx + 1.4, 12), (cx + 3.0, 18.5),
                (cx + 2.2, 24.5), (cx + 0.6, 20.0), (cx - 0.6, 15.5)]
        d.polygon(claw, fill=base)
        d.line([(cx + 1.4, 12), (cx + 3.0, 18.5), (cx + 2.2, 24.5)],
               fill=hi, width=1)
        d.line([(cx - 1.6, 12), (cx - 0.6, 15.5), (cx + 0.6, 20.0)],
               fill=dark, width=1)
    px = img.load()
    for y in range(32):
        for x in range(32):
            if px[x, y][3] == 255:
                px[x, y] = noisy(px[x, y][:3], noise[y][x]) + (255,)
    return img


def render_katar(rng):
    """Side view: five rebar prongs run along a forearm brace
    (two leather strap bands at the left) and extend past the
    fist to the right, fanning slightly."""
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    pal = PALS["steel"]
    rough = pal["rough"]
    noise = [[rng.randint(-rough, rough) for _ in range(32)]
             for _ in range(32)]
    rebar = (129, 129, 135, 255)
    rebar_d = (99, 99, 105, 255)
    hi = (183, 185, 191, 255)
    strap = (74, 55, 38, 255)
    strap_d = (55, 40, 26, 255)
    base_ys = (10.5, 13.5, 16.5, 19.5, 22.5)
    tip_ys = (8.5, 13.0, 16.5, 20.0, 24.0)
    # rebar shafts under the brace (left ends)
    for yb in base_ys:
        d.line([(2.5, yb), (11.5, yb)], fill=rebar, width=2)
        for rx in range(4, 11, 2):  # rebar ridge ticks
            d.point((rx, yb - 1), fill=rebar_d)
    # the five sharpened prongs
    for yb, yt in zip(base_ys, tip_ys):
        d.polygon([(11.0, yb - 0.9), (28.5, yt), (11.0, yb + 0.9)],
                  fill=rebar)
        d.line([(11.0, yb - 0.9), (28.5, yt)], fill=hi, width=1)
        d.line([(11.0, yb + 0.9), (28.5, yt)], fill=rebar_d, width=1)
    # strap bands fastening the brace (over the shafts)
    for x0 in (4.0, 8.0):
        d.rectangle([x0, 8.6, x0 + 2.6, 24.4], fill=strap)
        d.line([x0, 8.6, x0, 24.4], fill=strap_d, width=1)
        d.line([x0 + 2.6, 8.6, x0 + 2.6, 24.4], fill=strap_d, width=1)
    px = img.load()
    for y in range(32):
        for x in range(32):
            if px[x, y][3] == 255:
                px[x, y] = noisy(px[x, y][:3], noise[y][x]) + (255,)
    return img


def render(item_id, rng):
    if item_id == "bagh_nakha":
        return render_bagh_nakha(rng)
    if item_id == "knuckle_katar":
        return render_katar(rng)
    return render_bladed(item_id, SPECS[item_id], rng)


def main():
    imgs = {}
    for item_id in IDS:
        rng = random.Random(zlib.crc32(item_id.encode()))
        img = render(item_id, rng)
        assert img.size == (32, 32), item_id
        assert set(img.getchannel("A").getdata()) <= {0, 255}, item_id
        imgs[item_id] = img
        folder = os.path.join(BASE, item_id)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, item_id + ".png"))
        with open(os.path.join(folder, item_id + ".json"), "w") as f:
            json.dump([{"id": item_id, "fg": item_id, "bg": ""}], f,
                      indent=2)
            f.write("\n")
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS)
    cols, cell = 8, 66
    rows = (len(IDS) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, item_id in enumerate(IDS):
        big = imgs[item_id].resize((64, 64), Image.NEAREST)
        sheet.alpha_composite(big, ((i % cols) * cell + 1,
                                    (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item18.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item18.png {sheet.size}")


if __name__ == "__main__":
    main()
