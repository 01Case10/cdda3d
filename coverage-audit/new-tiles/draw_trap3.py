#!/usr/bin/env python3
"""Batch TRAP3 (Phase 2g.4 scope extension): 40 trap ids -- the final trap
batch. The portal / stairs / hazard tail.

Families, from the game data (all 40 entries read; copy-from resolved):
  * tr_light_on_res_8_SE / _SW / _SW_2 -- "overhead light", white ".",
    visibility 99: invisible mapgen triggers. Rendered with TRAP2's
    light-pool marker copied verbatim (self-contained), extended with the
    SW tick position; variants told by tick corner + pip count.
  * Portal devices (shipped tr_portal = dark tile + glowing ring is the
    family reference):
      - tr_swirling_vortex family (copy-from chain): green "#" vortices,
        visibility -1 (visible). tr_swirling_vortex is the canonical one;
        dungeon entrance / exit / teleporter ("elsewhere") are green,
        next_level is blue; told apart by a glyph badge (down chevron =
        entrance/next level, up chevron + daylight core = exit, star =
        elsewhere).
      - tr_ls_scarlet_tunnel_portal: concentric tunnel rings, scarlet.
      - tr_ls_sd_tunnel_portal: tunnel rings in pale string-dimension
        whites with string lines.
      - tr_ls_teleport_fixbot / _to_bridges / _from_bridges: invisible
        (visibility 1000, symbol " ") EOC teleport triggers -- subtle
        slate pads with small glyphs.
      - tr_microlab_portal_elevator_physics_shift: "microlab shifting
        elevator", magenta "." -- magenta portal pad with up/down arrows.
      - tr_portal_dungeon_how_dare_you_skip_the_real_teleporter and
        tr_portal_dungeon_to_labyrinth_teleporter_n/_s: "mysterious
        hallway", invisible -- full-tile corridor floor, told apart by
        glyph (amber "!" / N up-arrow / S down-arrow).
  * tr_nl_stairs_1_to_2 / _2_to_1 -- "mysterious staircase", invisible
    triggers: pale step glyphs with direction chevrons.
  * tr_lixa_fall_down / tr_lixa_climbing_fall_down -- "open air",
    dark_gray "#": dark fall shafts (climbing variant adds a rope).
  * tr_nl_chasm -- "sparkling chasm" (copy-from tr_swirling_vortex):
    jagged dark crack with sparkles.
  * Hazards drawn as what they are: light snare (brown "^" cord loop),
    mattress (white "0", benign), shifting halls (brown "." floor plates
    with opposed arrows), shadow (nameless "^" -- a shadow blob),
    sinkhole (brown "_"), overgrown slope, snake (nameless "^" -- a
    camouflaged coiled snake), trapdoor spiderden (green "#"),
    moonpool (blue "~" pool) and its exit (a "1D plane" -- a glowing
    line), spinner target (light_gray "%", benign), scattered monster
    teeth (light_cyan "_", caltrops), temple ledge/flood (light_gray
    "^"), temple toggle (nameless "^" -- a faint pressure plate),
    thick ice (cyan "/") vs thin ice (light_cyan "/"), void spider
    entry (green "#" -- a webbed rift with a spider).

PIL only, per-sprite seeded random.Random (crc32 of id). Alpha is strictly
{0, 255}: glow/translucency faked with pale opaque tones. Writes ONLY its
own 40 folders + preview_trap3.png. No other files touched.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "traps")
CREDITS = ("Drawn from the trap's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the "
           "tileset.\n")

IDS = [
    "tr_light_on_res_8_SE", "tr_light_on_res_8_SW", "tr_light_on_res_8_SW_2",
    "tr_light_snare", "tr_lixa_climbing_fall_down", "tr_lixa_fall_down",
    "tr_ls_scarlet_tunnel_portal", "tr_ls_sd_tunnel_portal",
    "tr_ls_teleport_fixbot", "tr_ls_teleport_from_bridges",
    "tr_ls_teleport_to_bridges", "tr_mattress",
    "tr_microlab_portal_elevator_physics_shift",
    "tr_microlab_shifting_hall", "tr_microlab_shifting_hall_2",
    "tr_nl_chasm", "tr_nl_stairs_1_to_2", "tr_nl_stairs_2_to_1",
    "tr_portal_dungeon_entrance", "tr_portal_dungeon_exit",
    "tr_portal_dungeon_how_dare_you_skip_the_real_teleporter",
    "tr_portal_dungeon_next_level", "tr_portal_dungeon_teleporter",
    "tr_portal_dungeon_to_labyrinth_teleporter_n",
    "tr_portal_dungeon_to_labyrinth_teleporter_s",
    "tr_shadow", "tr_sinkhole", "tr_slope_to_bosky_labyrinth",
    "tr_snake", "tr_spider", "tr_string_dimension_ka_moonpool",
    "tr_string_dimension_ka_moonpool_exit", "tr_swirling_vortex",
    "tr_target_spinner", "tr_teeth", "tr_temple_flood",
    "tr_temple_toggle", "tr_thick_ice", "tr_thin_ice",
    "tr_void_spider_entry",
]


# --------------------------------------------------------------------------
# Helpers (canvas / sparkle / taper copied from draw_trap2.py).
# --------------------------------------------------------------------------
def canvas():
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    return img, ImageDraw.Draw(img)


def sparkle(d, cx, cy, s, color):
    """Four-point star sparkle."""
    d.polygon([(cx, cy - s), (cx + s * 0.28, cy - s * 0.28),
               (cx + s, cy), (cx + s * 0.28, cy + s * 0.28),
               (cx, cy + s), (cx - s * 0.28, cy + s * 0.28),
               (cx - s, cy), (cx - s * 0.28, cy - s * 0.28)], fill=color)


def taper(d, x0, y0, w0, x1, y1, w1, fill):
    """Tapered limb/root from (x0,y0) width w0 to (x1,y1) width w1."""
    dx, dy = x1 - x0, y1 - y0
    ln = math.hypot(dx, dy) or 1.0
    px, py = -dy / ln, dx / ln
    d.polygon([(x0 + px * w0 / 2, y0 + py * w0 / 2),
               (x1 + px * w1 / 2, y1 + py * w1 / 2),
               (x1 - px * w1 / 2, y1 - py * w1 / 2),
               (x0 - px * w0 / 2, y0 - py * w0 / 2)], fill=fill)


# --------------------------------------------------------------------------
# Overhead-light marker: copied from draw_trap2.py (pattern + palette
# verbatim) and extended with the SW tick position + SW pips.
# --------------------------------------------------------------------------
def light_marker(tick_pos, pips):
    """Overhead-light trigger: pale pool of light, lamp dot, variant ticks."""
    img, d = canvas()
    d.ellipse([5, 7, 27, 25], fill=(224, 219, 184, 255))
    d.ellipse([5, 7, 27, 25], outline=(203, 198, 163, 255))
    d.ellipse([9, 10, 23, 22], fill=(240, 236, 205, 255))
    d.ellipse([14, 14, 18, 18], fill=(252, 250, 236, 255))
    d.point((16, 16), fill=(255, 255, 255, 255))
    tick = (124, 124, 117, 255)
    boxes = {"N": [14, 0.8, 18, 3.8], "S": [14, 28.2, 18, 31.2],
             "NE": [26.2, 0.8, 30.2, 4.8], "NW": [1.8, 0.8, 5.8, 4.8],
             "SE": [26.2, 27.2, 30.2, 31.2], "SW": [1.8, 27.2, 5.8, 31.2]}
    d.rectangle(boxes[tick_pos], fill=tick)
    # Pips sit just inside the ticked edge.
    for i in range(pips):
        if tick_pos == "N":
            x = 12.5 + i * 4.5 - (pips - 1) * 2.2
            d.rectangle([x, 5, x + 2, 7], fill=tick)
        elif tick_pos == "S":
            x = 12.5 + i * 4.5 - (pips - 1) * 2.2
            d.rectangle([x, 25, x + 2, 27], fill=tick)
        elif tick_pos == "SE":
            x = 21.5 + i * 3.4
            d.rectangle([x, 27.2, x + 2, 29.2], fill=tick)
        elif tick_pos == "SW":
            x = 8.5 + i * 3.6
            d.rectangle([x, 27.2, x + 2, 29.2], fill=tick)
    return img


# --------------------------------------------------------------------------
# Portal family.
# --------------------------------------------------------------------------
GREEN = ((168, 248, 178), (66, 196, 96), (18, 82, 38))
BLUE = ((165, 212, 250), (70, 132, 222), (20, 50, 112))
SCARLET = ((252, 176, 152), (216, 62, 48), (112, 22, 18))
STRING = ((246, 246, 250), (192, 197, 208), (92, 97, 112))
MAGENTA = ((246, 172, 236), (202, 72, 188), (102, 22, 92))


def chevron(d, cx, cy, s, color, up=False):
    """Bold chevron glyph centered at (cx, cy), half-size s."""
    if up:
        d.polygon([(cx - s, cy + s * 0.55), (cx, cy - s * 0.6),
                   (cx + s, cy + s * 0.55), (cx + s, cy + s * 1.15),
                   (cx, cy), (cx - s, cy + s * 1.15)], fill=color)
    else:
        d.polygon([(cx - s, cy - s * 0.55), (cx, cy + s * 0.6),
                   (cx + s, cy - s * 0.55), (cx + s, cy - s * 1.15),
                   (cx, cy), (cx - s, cy - s * 1.15)], fill=color)


def vortex(rng, tones, glyph=None, daylight_core=False):
    """Swirling vortex: dark disc, two-arm spiral, bright core, glyph badge.

    Echoes the shipped tr_portal (dark tile, glowing ring) with a spiral
    in the family tones. glyph: None | 'down' | 'up' | 'star'.
    """
    pale, mid, deep = tones
    img, d = canvas()
    cx = cy = 15.5
    # Dark speckled base disc (shipped tr_portal sits on near-black).
    d.ellipse([2.5, 2.5, 29.5, 29.5], fill=(13, 16, 15, 255))
    d.ellipse([2.5, 2.5, 29.5, 29.5], outline=(44, 52, 48, 255))
    for _ in range(30):
        a = rng.uniform(0, 2 * math.pi)
        rr = rng.uniform(9.0, 13.5)
        d.point((cx + rr * math.cos(a), cy + rr * math.sin(a)),
                fill=(30, 36, 33, 255))
    # Deep field, then the spiral arms.
    d.ellipse([5.5, 5.5, 26.5, 26.5], fill=deep + (255,))
    for arm in (0.0, math.pi):
        pts = []
        steps = 46
        for k in range(steps + 1):
            t = k / steps
            ang = arm + t * 3.9 * math.pi + rng.uniform(-0.02, 0.02)
            r = 1.0 + t * 9.6
            pts.append((cx + r * math.cos(ang), cy + r * math.sin(ang)))
        d.line(pts, fill=mid + (255,), width=3)
        d.line(pts, fill=pale + (255,), width=1)
    # Bright core.
    core = (250, 244, 198) if daylight_core else (240, 255, 242)
    d.ellipse([cx - 3.2, cy - 3.2, cx + 3.2, cy + 3.2], fill=core + (255,))
    d.ellipse([cx - 1.4, cy - 1.4, cx + 1.4, cy + 1.4],
              fill=(255, 255, 255, 255))
    # Glyph badge at the bottom rim.
    if glyph:
        bx, by = cx, 26.2
        d.ellipse([bx - 4.6, by - 4.6, bx + 4.6, by + 4.6],
                  fill=(10, 12, 11, 255), outline=mid + (255,))
        if glyph == "down":
            chevron(d, bx, by - 0.4, 2.6, (255, 255, 255, 255))
        elif glyph == "up":
            chevron(d, bx, by - 0.4, 2.6, (255, 255, 255, 255), up=True)
        elif glyph == "star":
            sparkle(d, bx, by, 3.0, (255, 255, 255, 255))
    return img


def tunnel(rng, tones, strings=False):
    """Concentric tunnel rings receding to a bright center."""
    pale, mid, deep = tones
    img, d = canvas()
    cx = cy = 15.5
    d.ellipse([2.5, 2.5, 29.5, 29.5], fill=(13, 14, 16, 255))
    for r, col, w in [(12.6, deep, 3), (10.4, mid, 3), (8.2, deep, 2),
                      (6.2, pale, 2), (4.4, mid, 2)]:
        d.ellipse([cx - r, cy - r * 0.82, cx + r, cy + r * 0.82],
                  outline=col + (255,), width=w)
    d.ellipse([cx - 2.6, cy - 2.1, cx + 2.6, cy + 2.1],
              fill=(255, 252, 246, 255))
    if strings:
        # String-dimension strings: fine horizontal lines across the rings.
        for y in (8.5, 12.5, 19.5, 23.5):
            d.line([(4.5, y), (27.5, y)], fill=(210, 214, 224, 255))
    for _ in range(6):
        a = rng.uniform(0, 2 * math.pi)
        d.point((cx + 12.8 * math.cos(a), cy + 10.5 * math.sin(a)),
                fill=pale + (255,))
    return img


def teleport_pad(rng, glyph):
    """Subtle slate pad for the invisible tr_ls_teleport_* triggers."""
    img, d = canvas()
    slate = (56, 60, 68, 255)
    slate_e = (88, 94, 106, 255)
    mark = (206, 211, 219, 255)
    d.ellipse([5, 7, 27, 25], fill=slate)
    d.ellipse([5, 7, 27, 25], outline=slate_e)
    d.ellipse([8.5, 10, 23.5, 22], outline=slate_e)
    if glyph == "fixbot":
        # Little bot head: square, antenna, eyes; wrench dot beside it.
        d.rectangle([12, 11, 20, 19], outline=mark, width=1)
        d.line([(16, 11), (16, 8.5)], fill=mark)
        d.point((16, 8), fill=mark)
        d.point((14, 15), fill=mark)
        d.point((18, 15), fill=mark)
        d.line([(21.5, 20.5), (24, 23)], fill=mark)
        d.point((24.5, 23.5), fill=mark)
    elif glyph == "to_bridges":
        # Bridge deck lines at top; arrow rises toward them.
        d.line([(9, 8.5), (23, 8.5)], fill=mark)
        d.line([(9, 11), (23, 11)], fill=slate_e)
        d.polygon([(16, 13.5), (19, 17.5), (17.2, 17.5), (17.2, 22),
                   (14.8, 22), (14.8, 17.5), (13, 17.5)], fill=mark)
    elif glyph == "from_bridges":
        # Bridge deck lines at top; arrow descends away from them.
        d.line([(9, 8.5), (23, 8.5)], fill=mark)
        d.line([(9, 11), (23, 11)], fill=slate_e)
        d.polygon([(16, 21.5), (13, 17.5), (14.8, 17.5), (14.8, 13),
                   (17.2, 13), (17.2, 17.5), (19, 17.5)], fill=mark)
    return img


def hallway(rng, glyph):
    """'Mysterious hallway': full-tile corridor floor with a glyph mark."""
    img, d = canvas()
    d.rectangle([0, 0, 32, 32], fill=(54, 54, 60, 255))
    for _ in range(40):  # worn stone speckle
        d.point((rng.uniform(1, 31), rng.uniform(1, 31)),
                fill=(64, 64, 70, 255) if rng.random() < 0.5
                else (46, 46, 52, 255))
    # Receding corridor: converging walls, dark doorway at the vanishing end.
    d.polygon([(0, 0), (11, 9), (11, 23), (0, 32)], fill=(44, 44, 50, 255))
    d.polygon([(32, 0), (21, 9), (21, 23), (32, 32)],
              fill=(44, 44, 50, 255))
    d.polygon([(0, 0), (32, 0), (21, 9), (11, 9)], fill=(48, 48, 54, 255))
    d.polygon([(0, 32), (32, 32), (21, 23), (11, 23)],
              fill=(40, 40, 46, 255))
    d.rectangle([11, 9, 21, 23], fill=(16, 15, 20, 255))
    d.rectangle([11, 9, 21, 23], outline=(96, 92, 84, 255))
    d.line([(16, 23), (16, 12)], fill=(70, 60, 66, 255))
    if glyph == "N":
        chevron(d, 16, 27.6, 3.0, (240, 240, 244, 255), up=True)
    elif glyph == "S":
        chevron(d, 16, 27.6, 3.0, (240, 240, 244, 255))
    elif glyph == "!":
        # The skip-guard's own mark: an amber exclamation in a ring.
        d.ellipse([11.5, 23.5, 20.5, 31.5], outline=(232, 176, 64, 255),
                  width=2)
        d.rectangle([15.2, 25.6, 16.8, 28.6], fill=(232, 176, 64, 255))
        d.point((16, 30), fill=(232, 176, 64, 255))
    return img


def microlab_elevator(rng):
    """Magenta portal pad with elevator up/down arrows."""
    pale, mid, deep = MAGENTA
    img, d = canvas()
    cx = cy = 15.5
    d.ellipse([3.5, 4.5, 28.5, 27.5], fill=deep + (255,))
    d.ellipse([3.5, 4.5, 28.5, 27.5], outline=mid + (255,), width=2)
    d.ellipse([7, 8, 25, 24], outline=pale + (255,))
    # Paired elevator arrows.
    d.polygon([(12.6, 10.5), (15.4, 14.5), (13.9, 14.5), (13.9, 21),
               (11.9, 21), (11.9, 14.5), (10.5, 14.5)],
              fill=(255, 255, 255, 255))
    d.polygon([(19.4, 21.5), (16.6, 17.5), (18.1, 17.5), (18.1, 11),
               (20.1, 11), (20.1, 17.5), (21.5, 17.5)],
              fill=(255, 255, 255, 255))
    sparkle(d, 6.5, 7.5, 1.8, pale + (255,))
    sparkle(d, 25.5, 24.5, 1.8, pale + (255,))
    return img


def shifting_hall(rng, variant):
    """Brown floor plate with opposed shift arrows; variant 2 adds pips."""
    img, d = canvas()
    plate = (130, 103, 73, 255)
    plate_d = (104, 80, 54, 255)
    arrow = (234, 222, 200, 255)
    d.rounded_rectangle([3.5, 8.5, 28.5, 23.5], radius=3, fill=plate)
    d.rounded_rectangle([3.5, 8.5, 28.5, 23.5], radius=3, outline=plate_d)
    for _ in range(14):
        d.point((rng.uniform(5, 27), rng.uniform(10, 22)), fill=plate_d)
    # Opposed horizontal arrows: the hall shifts both ways.
    d.polygon([(9, 12.6), (19, 12.6), (19, 10.9), (23.5, 14.2), (19, 17.5),
               (19, 15.8), (9, 15.8)], fill=arrow)
    d.polygon([(23, 17.4), (13, 17.4), (13, 15.7), (8.5, 19), (13, 22.3),
               (13, 20.6), (23, 20.6)], fill=arrow)
    if variant == 2:
        for i in range(2):
            x = 13.5 + i * 4
            d.rectangle([x, 26, x + 2.4, 28.4], fill=plate)
    return img


# --------------------------------------------------------------------------
# Stairs / shafts / chasm.
# --------------------------------------------------------------------------
def stairs(rng, down):
    """Pale mysterious-staircase glyph: steps + direction chevron."""
    img, d = canvas()
    step = (196, 196, 200, 255)
    step_d = (148, 148, 154, 255)
    # Four steps; 1_to_2 descends left-to-right, 2_to_1 ascends.
    for i in range(4):
        x = 4 + i * 6
        y = (8 + i * 4.4) if down else (22 - i * 4.4)
        d.rectangle([x, y, x + 7, y + 3.2], fill=step)
        d.rectangle([x, y + 3.2, x + 7, y + 4.6], fill=step_d)
    bx, by = (24.5, 8.5) if down else (24.5, 23.5)
    d.ellipse([bx - 4.6, by - 4.6, bx + 4.6, by + 4.6],
              fill=(24, 24, 28, 255), outline=step)
    chevron(d, bx, by - 0.4, 2.6, (255, 255, 255, 255), up=not down)
    return img


def fall_shaft(rng, climbing):
    """'Open air': a dark fall shaft. The climbing variant adds a rope."""
    img, d = canvas()
    d.rectangle([2.5, 2.5, 29.5, 29.5], fill=(74, 74, 78, 255))
    d.rectangle([2.5, 2.5, 29.5, 29.5], outline=(52, 52, 56, 255))
    for _ in range(20):
        d.point((rng.uniform(3, 29), rng.uniform(3, 29)),
                fill=(64, 64, 68, 255))
    d.rectangle([6, 6, 26, 26], fill=(13, 13, 15, 255))
    d.ellipse([9, 9, 23, 23], fill=(5, 5, 6, 255))
    if climbing:
        # Rope hanging down the left wall, with rungs.
        rope = (168, 138, 92, 255)
        d.line([(8.6, 3), (8.6, 22)], fill=rope)
        d.line([(10.4, 3), (10.4, 22)], fill=rope)
        for y in (7, 11, 15, 19):
            d.line([(8.2, y), (10.8, y)], fill=rope)
    # Down chevron: the way is down.
    chevron(d, 16, 20.5, 3.4, (128, 128, 134, 255))
    return img


def chasm(rng):
    """'Sparkling chasm': a jagged black crack, sparkles on its rim."""
    img, d = canvas()
    rim = (62, 62, 66, 255)
    deep = (8, 8, 10, 255)
    top = [(0, 13), (5, 11), (10, 14), (15, 10.5), (20, 13.5), (25, 11),
           (32, 13)]
    bot = [(32, 20), (26, 22.5), (21, 19.5), (16, 23), (11, 20),
           (6, 22.5), (0, 19.5)]
    d.polygon([(x, y - 2.2) for x, y in top] + [(x, y + 2.2) for x, y in bot],
              fill=rim)
    d.polygon(top + bot, fill=deep)
    # Branch cracks.
    d.line([(10, 13), (7, 7), (9, 3)], fill=rim, width=2)
    d.line([(21, 20), (24, 26), (22, 30)], fill=rim, width=2)
    # The sparkle that gives it its name.
    for (x, y, s) in [(7, 9, 2.2), (18, 8, 1.8), (26, 24, 2.2),
                      (13, 25, 1.6)]:
        sparkle(d, x, y, s, (250, 236, 170, 255))
    for _ in range(6):
        d.point((rng.uniform(2, 30), rng.uniform(4, 28)),
                fill=(235, 240, 250, 255))
    return img


# --------------------------------------------------------------------------
# Hazards and misc devices.
# --------------------------------------------------------------------------
def light_snare(rng):
    """Brown cord snare: a noose loop on the ground, line to a bent stick."""
    img, d = canvas()
    cord = (128, 90, 52, 255)
    cord_d = (100, 68, 38, 255)
    # The noose loop lying on the ground.
    d.ellipse([9, 12, 27, 27], outline=cord, width=3)
    d.ellipse([13.5, 16, 22.5, 23], outline=cord_d, width=2)
    d.ellipse([16.8, 12.5, 20.2, 15.8], fill=cord_d)  # slip knot
    # Running line up to the bent sapling anchor at the corner.
    d.line([(18, 13), (8, 5)], fill=cord, width=2)
    d.arc([1, -6, 15, 12], start=40, end=130, fill=(110, 96, 58, 255),
          width=3)
    d.line([(5, 4), (3, 9)], fill=cord_d, width=2)
    for _ in range(8):  # leaf litter camouflage
        d.point((rng.uniform(4, 28), rng.uniform(16, 30)),
                fill=(104, 110, 62, 255))
    return img


def mattress(rng):
    """Plain white mattress (benign; the down mattress in TRAP2 is quilted,
    this one is smooth with piping and a '0' oval echo of its symbol)."""
    img, d = canvas()
    d.rounded_rectangle([5, 8, 30, 29], radius=4, fill=(148, 146, 138, 255))
    d.rounded_rectangle([3, 5, 28, 26], radius=4, fill=(238, 236, 229, 255))
    d.rounded_rectangle([3, 22, 28, 26], radius=4, fill=(224, 222, 214, 255))
    d.rounded_rectangle([3, 5, 28, 26], radius=4,
                        outline=(203, 201, 192, 255))
    d.rounded_rectangle([5.5, 7.5, 25.5, 23.5], radius=3,
                        outline=(214, 212, 203, 255))
    # The "0" of its data symbol, as a faint oval ring at the center.
    d.ellipse([12, 10.5, 19, 20.5], outline=(206, 204, 196, 255), width=2)
    return img


def shadow(rng):
    """The nameless shadow trap: an amorphous shadow with two faint eyes."""
    img, d = canvas()
    blob = (26, 26, 32, 255)
    core = (14, 14, 18, 255)
    d.ellipse([7, 10, 26, 27], fill=blob)
    d.ellipse([4, 15, 15, 25], fill=blob)
    d.ellipse([18, 6, 27, 17], fill=blob)
    d.ellipse([11, 13, 22, 24], fill=core)
    # Wispy tendrils.
    for (x0, y0, x1, y1) in [(9, 24, 5, 30), (16, 26, 15, 31),
                             (23, 24, 27, 29)]:
        taper(d, x0, y0, 2.6, x1, y1, 0.7, blob)
    # Two faint eyes in the dark.
    d.point((13.5, 16.5), fill=(168, 172, 190, 255))
    d.point((19.5, 16.5), fill=(168, 172, 190, 255))
    return img


def sinkhole(rng):
    """Brown sinkhole: crumbling earth rings around a jagged dark hole."""
    img, d = canvas()
    dirt = (118, 92, 60, 255)
    dirt_d = (94, 70, 44, 255)
    dirt_l = (140, 112, 78, 255)
    d.ellipse([1.5, 6, 30.5, 28], fill=dirt)
    for _ in range(30):
        x = rng.uniform(3, 29)
        y = rng.uniform(8, 27)
        d.point((x, y), fill=dirt_d if rng.random() < 0.5 else dirt_l)
    d.ellipse([5.5, 9.5, 26.5, 25], fill=dirt_d)
    hole = [(10, 12), (15, 10.5), (20, 12), (23, 15), (21.5, 19),
            (17, 21.5), (12.5, 20.5), (9.5, 16.5)]
    d.polygon(hole, fill=(26, 20, 14, 255))
    d.polygon([(13, 14), (18, 13.5), (19.5, 17), (16, 19), (13.5, 17.5)],
              fill=(12, 9, 6, 255))
    # Cracks radiating into the rim.
    d.line([(11, 11.5), (6, 8), (3.5, 8.5)], fill=dirt_d)
    d.line([(21, 12), (26, 8.5), (29, 9.5)], fill=dirt_d)
    d.line([(16, 21.8), (15, 26), (17, 28.5)], fill=dirt_d)
    return img


def slope(rng):
    """'Overgrown slope': a vegetated diagonal slope down to the labyrinth."""
    img, d = canvas()
    d.rectangle([0, 0, 32, 32], fill=(50, 80, 42, 255))
    # The slope face, descending left-to-right.
    d.polygon([(0, 9), (32, 20), (32, 32), (0, 32)], fill=(112, 97, 66, 255))
    d.polygon([(0, 9), (32, 20), (32, 23), (0, 12)], fill=(92, 79, 54, 255))
    # Foliage: leaf blobs above and tufts breaking over the face.
    for _ in range(46):
        x = rng.uniform(1, 31)
        y = rng.uniform(1, 30)
        on_face = y > 9 + x * 0.34
        c = (70, 112, 54, 255) if rng.random() < 0.6 else (92, 134, 66, 255)
        r = rng.uniform(1.0, 2.4)
        if on_face and rng.random() < 0.45:
            d.ellipse([x - r, y - r, x + r, y + r], fill=c)
        elif not on_face:
            d.ellipse([x - r, y - r, x + r, y + r], fill=c)
    for _ in range(6):  # stones on the slope
        x = rng.uniform(4, 28)
        y = rng.uniform(16, 29)
        d.ellipse([x - 1.4, y - 1, x + 1.4, y + 1], fill=(140, 138, 130, 255))
    return img


def snake(rng):
    """The hidden snake trap: a camouflaged coiled snake, head up."""
    img, d = canvas()
    body = (112, 106, 64, 255)
    body_d = (88, 82, 48, 255)
    belly = (150, 142, 96, 255)
    # Coils: stacked ellipse bands.
    d.ellipse([5, 17, 27, 28], fill=body_d)
    d.ellipse([5, 17, 27, 28], outline=body, width=3)
    d.ellipse([7.5, 13.5, 24.5, 24], fill=body)
    d.ellipse([7.5, 13.5, 24.5, 24], outline=body_d, width=2)
    d.ellipse([10, 10.5, 22, 20], fill=body_d)
    d.ellipse([10, 10.5, 22, 20], outline=belly, width=2)
    # Scale speckle.
    for _ in range(26):
        a = rng.uniform(0, 2 * math.pi)
        rr = rng.uniform(4.0, 10.0)
        x, y = 16 + rr * math.cos(a), 19 + rr * 0.62 * math.sin(a)
        if 4 < x < 28 and 12 < y < 28:
            d.point((x, y), fill=body_d if rng.random() < 0.5 else belly)
    # Neck rising, head with eye and forked tongue.
    taper(d, 20, 13, 3.4, 23.5, 7.5, 2.4, body)
    d.ellipse([21, 4.5, 26.5, 9], fill=body)
    d.point((24.6, 6.4), fill=(20, 18, 12, 255))
    d.line([(26.3, 6.8), (29.5, 5.8)], fill=(178, 52, 48, 255))
    d.line([(29.5, 5.8), (31, 4.6)], fill=(178, 52, 48, 255))
    d.line([(29.5, 5.8), (31.2, 6.6)], fill=(178, 52, 48, 255))
    return img


def spider_den(rng):
    """'Trapdoor spiderden': a hinged trapdoor in green-speckled earth,
    web strands at the corners."""
    img, d = canvas()
    earth = (96, 86, 62, 255)
    earth_d = (76, 67, 46, 255)
    d.ellipse([2, 8, 30, 28], fill=earth)
    for _ in range(22):
        x = rng.uniform(4, 28)
        y = rng.uniform(10, 27)
        d.point((x, y), fill=earth_d if rng.random() < 0.55
                else (96, 138, 74, 255))  # data color green speckle
    # The trapdoor: a near-circular lid with a hinge and radial seam.
    d.ellipse([8, 7, 24, 23], fill=(124, 110, 80, 255))
    d.ellipse([8, 7, 24, 23], outline=earth_d, width=2)
    d.arc([8, 7, 24, 23], start=300, end=120, fill=(146, 130, 96, 255),
          width=2)
    d.rectangle([14.6, 5.6, 17.4, 8.6], fill=earth_d)  # hinge
    d.line([(16, 8.6), (16, 21)], fill=earth_d)
    # Web strands spun from the rim to the corners.
    web = (214, 218, 208, 255)
    for (x, y) in [(2, 3), (30, 3), (2, 29), (30, 29)]:
        d.line([(16, 15), (x, y)], fill=web)
    d.arc([10, 9, 22, 21], start=140, end=220, fill=web)
    d.arc([4, 3, 28, 27], start=30, end=80, fill=web)
    return img


def moonpool(rng):
    """'Strange moonpool': a deep blue pool, '~' waves, a moon reflection."""
    img, d = canvas()
    deep = (22, 50, 108, 255)
    mid = (38, 82, 152, 255)
    lite = (150, 190, 236, 255)
    d.ellipse([1.5, 4.5, 30.5, 27.5], fill=deep)
    d.ellipse([1.5, 4.5, 30.5, 27.5], outline=(16, 36, 82, 255))
    d.ellipse([4.5, 7, 27.5, 25], fill=mid)
    # Wave lines (data symbol "~").
    for wy in (12.5, 17, 21.5):
        pts = [(6 + t, wy + 1.1 * math.sin(t / 2.4 + wy))
               for t in range(21)]
        d.line(pts, fill=lite)
    # The moon, reflected: pale disc and its broken reflection.
    d.ellipse([7.5, 6.8, 12.5, 11.2], fill=(238, 242, 250, 255))
    for i, w in enumerate((3.6, 2.6, 1.6)):
        y = 13.5 + i * 2.2
        d.line([(10 - w / 2, y), (10 + w / 2, y)], fill=(220, 230, 246, 255))
    sparkle(d, 23, 9.5, 1.8, (230, 240, 252, 255))
    return img


def moonpool_exit(rng):
    """'1D plane': a single glowing blue line -- a plane of one dimension."""
    img, d = canvas()
    # Faint vertical shimmer where the plane cuts the world.
    for x in (8, 13, 19, 24):
        d.line([(x, 7), (x, 25)], fill=(44, 74, 118, 255))
    d.line([(3, 16), (29, 16)], fill=(70, 130, 210, 255), width=3)
    d.line([(3, 16), (29, 16)], fill=(235, 246, 255, 255), width=1)
    d.ellipse([1.8, 14.8, 4.2, 17.2], fill=(150, 195, 245, 255))
    d.ellipse([27.8, 14.8, 30.2, 17.2], fill=(150, 195, 245, 255))
    sparkle(d, 16, 16, 2.4, (200, 226, 252, 255))
    return img


def target_spinner(rng):
    """'Spinner target' (benign): a four-paddle spinning target on a stand."""
    img, d = canvas()
    gray = (198, 200, 203, 255)
    gray_d = (150, 152, 156, 255)
    red = (192, 62, 52, 255)
    # Stand and pole.
    d.polygon([(11, 29), (21, 29), (17.5, 24), (14.5, 24)], fill=gray_d)
    d.rectangle([15.2, 18, 16.8, 25], fill=gray_d)
    # Four paddles on the diagonals (the "%" of it), two tipped red.
    cx = cy = 12.5
    for k in range(4):
        a = math.radians(45 + k * 90)
        ex, ey = cx + 8.6 * math.cos(a), cy + 8.6 * math.sin(a)
        px, py = -math.sin(a), math.cos(a)
        w = 3.1
        d.polygon([(cx + 2.2 * math.cos(a) + px * w, cy + 2.2 * math.sin(a) + py * w),
                   (ex + px * w, ey + py * w), (ex - px * w, ey - py * w),
                   (cx + 2.2 * math.cos(a) - px * w, cy + 2.2 * math.sin(a) - py * w)],
                  fill=gray)
        tip = red if k in (0, 2) else gray_d
        d.ellipse([ex - 2.5, ey - 2.5, ex + 2.5, ey + 2.5], fill=tip)
    d.ellipse([cx - 2.6, cy - 2.6, cx + 2.6, cy + 2.6], fill=gray_d)
    d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1], fill=(84, 86, 90, 255))
    return img


def teeth(rng):
    """'Scattered monster teeth' (light_cyan): big pale fangs, scattered."""
    img, d = canvas()
    fang = (228, 246, 250, 255)
    fang_d = (156, 204, 214, 255)
    root = (196, 178, 150, 255)
    spots = [(7, 8, 0.5), (16, 6, 1.9), (25, 9, 3.6), (10, 17, 2.8),
             (21, 16, 0.1), (6, 25, 4.4), (15, 24, 5.5), (26, 24, 1.2)]
    for (x, y, a) in spots:
        s = rng.uniform(3.4, 4.6)
        tip = (x + s * math.cos(a), y + s * math.sin(a))
        px, py = -math.sin(a), math.cos(a)
        b1 = (x + 1.9 * px, y + 1.9 * py)
        b2 = (x - 1.9 * px, y - 1.9 * py)
        d.ellipse([x - 2.6, y + 1.2, x + 2.6, y + 2.8],
                  fill=(120, 130, 128, 255))  # ground shadow
        d.polygon([b1, tip, b2], fill=fang)
        d.line([b1, tip], fill=fang_d)
        d.line([b1, b2], fill=root, width=2)
    return img


def temple_flood(rng):
    """'Ledge' (temple flood): a carved stone ledge above rising water."""
    img, d = canvas()
    stone = (152, 150, 144, 255)
    stone_d = (118, 116, 110, 255)
    water = (50, 94, 140, 255)
    wave = (140, 182, 212, 255)
    # Flood water below.
    d.rectangle([0, 13, 32, 32], fill=water)
    for wy in (19, 24.5, 29.5):
        pts = [(t, wy + 1.0 * math.sin(t / 2.6 + wy)) for t in range(0, 33, 2)]
        d.line(pts, fill=wave)
    # The ledge: stone slab with block seams and a carved lip.
    d.rectangle([0, 0, 32, 14], fill=stone)
    d.rectangle([0, 11.5, 32, 14], fill=stone_d)
    for x in (8, 16, 24):
        d.line([(x, 0), (x, 11.5)], fill=stone_d)
    d.line([(0, 5.5), (32, 5.5)], fill=stone_d)
    for x in (4, 12, 20, 28):  # hanging drip points on the lip
        d.polygon([(x - 1.2, 14), (x + 1.2, 14), (x, 16.4)], fill=stone_d)
    return img


def temple_toggle(rng):
    """The nameless temple toggle: a faint stone pressure plate."""
    img, d = canvas()
    plate = (142, 140, 134, 255)
    plate_d = (112, 110, 104, 255)
    for _ in range(10):
        d.point((rng.uniform(4, 28), rng.uniform(4, 28)), fill=plate_d)
    d.rounded_rectangle([6.5, 6.5, 25.5, 25.5], radius=2, fill=plate)
    d.rounded_rectangle([6.5, 6.5, 25.5, 25.5], radius=2, outline=plate_d)
    d.rounded_rectangle([10, 10, 22, 22], radius=1, outline=plate_d)
    # Temple glyph: a triangle eye.
    d.polygon([(16, 12.6), (19.6, 18.8), (12.4, 18.8)], outline=plate_d)
    d.ellipse([15, 16.2, 17, 18.2], fill=plate_d)
    return img


def ice(rng, thick):
    """Full-tile ice. Thick: deep cyan, calm, faint cracks. Thin: pale,
    crack web with dark water showing through."""
    img, d = canvas()
    if thick:
        base = (62, 178, 204, 255)
        patch = (84, 194, 216, 255)
        crack_c = (196, 240, 248, 255)
    else:
        base = (172, 230, 240, 255)
        patch = (190, 238, 246, 255)
        crack_c = (250, 254, 255, 255)
    d.rectangle([0, 0, 32, 32], fill=base)
    for _ in range(7):
        x = rng.uniform(0, 26)
        y = rng.uniform(0, 26)
        d.ellipse([x, y, x + rng.uniform(5, 10), y + rng.uniform(3, 6)],
                  fill=patch)
    # Glossy sheen band.
    d.polygon([(2, 32), (14, 0), (21, 0), (9, 32)],
              fill=(214, 246, 250, 255) if thick else (240, 252, 253, 255))

    def crack(x, y, ang, ln):
        pts = [(x, y)]
        for _ in range(3):
            ang += rng.uniform(-0.5, 0.5)
            ln_seg = ln / 3
            x += ln_seg * math.cos(ang)
            y += ln_seg * math.sin(ang)
            pts.append((x, y))
        d.line(pts, fill=crack_c)

    if thick:
        crack(6, 9, 0.4, 11)
        crack(19, 24, 2.6, 9)
    else:
        # Dark water blotches first, then the crack web over them.
        for (x, y, r) in [(9, 10, 3.4), (22, 20, 4.0), (14, 27, 2.8)]:
            d.ellipse([x - r, y - r * 0.7, x + r, y + r * 0.7],
                      fill=(34, 84, 102, 255))
        for _ in range(6):
            crack(rng.uniform(3, 26), rng.uniform(3, 26),
                  rng.uniform(0, 6.28), rng.uniform(8, 14))
    return img


def void_spider(rng):
    """Void spider entry: a dark rift rimmed in green, webbed, a spider
    waiting at its heart."""
    img, d = canvas()
    web = (196, 226, 200, 255)
    # Web strands to the corners first (behind the rift).
    for (x, y) in [(1, 1), (31, 1), (1, 31), (31, 31), (16, 0), (16, 32)]:
        d.line([(16, 16), (x, y)], fill=web)
    d.arc([6, 6, 26, 26], start=20, end=150, fill=web)
    d.arc([6, 6, 26, 26], start=200, end=330, fill=web)
    # The rift: a jagged vertical tear.
    tear = [(13.5, 5), (17, 8), (15.2, 12), (18, 16), (15.4, 20),
            (17.5, 24), (14.5, 27), (12.5, 23), (14, 19), (11.8, 15),
            (14.2, 11), (12, 7.5)]
    d.polygon(tear, fill=(7, 8, 9, 255))
    d.line(tear + [tear[0]], fill=(66, 196, 96, 255), width=1)
    # The spider: body, head, eight legs gripping the rim.
    leg = (20, 26, 20, 255)
    for (x0, y0, x1, y1, x2, y2) in [
            (14.5, 14, 10, 11, 7.5, 12.5), (17.5, 14, 22, 11, 24.5, 12.5),
            (14.2, 16, 9.5, 16, 7, 15), (17.8, 16, 22.5, 16, 25, 15),
            (14.2, 18, 9.5, 20, 7.5, 19), (17.8, 18, 22.5, 20, 24.5, 19),
            (14.5, 20, 10.5, 23, 8.5, 21.5), (17.5, 20, 21.5, 23, 23.5, 21.5)]:
        d.line([(x0, y0), (x1, y1), (x2, y2)], fill=leg, width=1)
    d.ellipse([13.6, 15.2, 18.4, 20.4], fill=(16, 20, 16, 255))
    d.ellipse([14.6, 12.8, 17.4, 15.4], fill=(16, 20, 16, 255))
    d.point((15.2, 14), fill=(120, 240, 140, 255))
    d.point((16.8, 14), fill=(120, 240, 140, 255))
    return img


# --------------------------------------------------------------------------
RENDERERS = {
    "tr_light_on_res_8_SE": lambda rng: light_marker("SE", 0),
    "tr_light_on_res_8_SW": lambda rng: light_marker("SW", 0),
    "tr_light_on_res_8_SW_2": lambda rng: light_marker("SW", 2),
    "tr_light_snare": light_snare,
    "tr_lixa_climbing_fall_down": lambda rng: fall_shaft(rng, climbing=True),
    "tr_lixa_fall_down": lambda rng: fall_shaft(rng, climbing=False),
    "tr_ls_scarlet_tunnel_portal": lambda rng: tunnel(rng, SCARLET),
    "tr_ls_sd_tunnel_portal": lambda rng: tunnel(rng, STRING, strings=True),
    "tr_ls_teleport_fixbot": lambda rng: teleport_pad(rng, "fixbot"),
    "tr_ls_teleport_from_bridges":
        lambda rng: teleport_pad(rng, "from_bridges"),
    "tr_ls_teleport_to_bridges": lambda rng: teleport_pad(rng, "to_bridges"),
    "tr_mattress": mattress,
    "tr_microlab_portal_elevator_physics_shift": microlab_elevator,
    "tr_microlab_shifting_hall": lambda rng: shifting_hall(rng, 1),
    "tr_microlab_shifting_hall_2": lambda rng: shifting_hall(rng, 2),
    "tr_nl_chasm": chasm,
    "tr_nl_stairs_1_to_2": lambda rng: stairs(rng, down=True),
    "tr_nl_stairs_2_to_1": lambda rng: stairs(rng, down=False),
    "tr_portal_dungeon_entrance":
        lambda rng: vortex(rng, GREEN, glyph="down"),
    "tr_portal_dungeon_exit":
        lambda rng: vortex(rng, GREEN, glyph="up", daylight_core=True),
    "tr_portal_dungeon_how_dare_you_skip_the_real_teleporter":
        lambda rng: hallway(rng, "!"),
    "tr_portal_dungeon_next_level":
        lambda rng: vortex(rng, BLUE, glyph="down"),
    "tr_portal_dungeon_teleporter":
        lambda rng: vortex(rng, GREEN, glyph="star"),
    "tr_portal_dungeon_to_labyrinth_teleporter_n":
        lambda rng: hallway(rng, "N"),
    "tr_portal_dungeon_to_labyrinth_teleporter_s":
        lambda rng: hallway(rng, "S"),
    "tr_shadow": shadow,
    "tr_sinkhole": sinkhole,
    "tr_slope_to_bosky_labyrinth": slope,
    "tr_snake": snake,
    "tr_spider": spider_den,
    "tr_string_dimension_ka_moonpool": moonpool,
    "tr_string_dimension_ka_moonpool_exit": moonpool_exit,
    "tr_swirling_vortex": lambda rng: vortex(rng, GREEN),
    "tr_target_spinner": target_spinner,
    "tr_teeth": teeth,
    "tr_temple_flood": temple_flood,
    "tr_temple_toggle": temple_toggle,
    "tr_thick_ice": lambda rng: ice(rng, thick=True),
    "tr_thin_ice": lambda rng: ice(rng, thick=False),
    "tr_void_spider_entry": void_spider,
}


def main():
    assert len(IDS) == 40, len(IDS)
    assert set(RENDERERS) == set(IDS), (
        set(IDS) ^ set(RENDERERS))
    for tid in IDS:
        rng = random.Random(zlib.crc32(tid.encode()))
        img = RENDERERS[tid](rng)
        alphas = set(img.getchannel("A").getdata())
        assert alphas <= {0, 255}, (tid, alphas)
        folder = os.path.join(BASE, tid)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, tid + ".png"))
        with open(os.path.join(folder, tid + ".json"), "w") as f:
            json.dump([{"id": tid, "fg": tid, "bg": ""}], f, indent=2)
            f.write("\n")
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS)
        print("wrote", tid)

    # Contact sheet: 8 cols x 5 rows in assignment order, sprites at 2x.
    cell, gap, cols = 64, 3, 8
    rows = (len(IDS) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * (cell + gap) + gap,
                               rows * (cell + gap) + gap), (30, 32, 36, 255))
    for i, tid in enumerate(IDS):
        img = Image.open(os.path.join(BASE, tid, tid + ".png"))
        img = img.resize((cell, cell), Image.NEAREST)
        x = gap + (i % cols) * (cell + gap)
        y = gap + (i // cols) * (cell + gap)
        sheet.alpha_composite(img, (x, y))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_trap3.png")
    sheet.save(out)
    print("wrote preview_trap3.png", sheet.size, "| total ids:", len(IDS))


if __name__ == "__main__":
    main()
