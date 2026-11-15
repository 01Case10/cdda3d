#!/usr/bin/env python3
"""
Draw the missing alien / Exodii furniture sprites from their game descriptions.

Sources (fork-src/cdda/data/json/furniture_and_terrain/furniture-alien.json
and furniture_alien_meadow.json, CDDA 6a3e59e0) -- 19 ids in those files are
missing from UltiCa in the shipped universe (coverage-audit/missing.json,
category "furniture"); the files' other ids resolve via existing tile
entries or looks_like and stay out of this batch:

  f_alien_scar            "scarred lump" -- "A pile of nondescript alien
                           flesh, twitching and belching strange gases out
                           of injured orifices."  symbol {, color green
  f_exodii_portal_enclosure
                          "portal enclosure" -- "A heavy metal ring attached
                           to dozens of wires, charred by residue that gives
                           off an acrid, unfamiliar smell."  Q, light_gray
  f_exodii_floor_portal_enclosure / f_string_dimension_floor_portal_enclosure
                          "metal ring" -- "A heavy metal ring sitting on the
                           floor, surrounding a spot charred by residue..."
                           O, light_gray (the two entries are verbatim
                           identical in game data, so -- like the string
                           dimension's identical planes -- they share one
                           drawing and one seed)
  f_exodii_locator_antenna
                          "locator antenna" -- "A tall metal pole solidly
                           fixed to the roof... a small set of holes near
                           its base."  Z, light_gray
  f_crystalline_flower    "A crystalline structure that resembles a plant."
                           f, light_green, TINY
  f_impaled_officer       "A dead body encased in high-tech armor hanging in
                           the air... impaled [by] the strange glowing
                           crystal that has seeped out of the ground."
                           !, red
  f_impaled_soldier       same, "military armor".  !, brown
  f_empty_spike           "A strange otherworldly spike that refracts light
                           in strange ways.  The body no longer rests on it."
                           !, black
  f_string_dimension_pylon
                          "humming column" -- "A floor-to-ceiling cylinder
                           wrapped neatly and tightly in thick copper wire...
                           cables connected to the base."  O, red
  f_string_dimension_pylon_inert
                          "inert column" -- the same cylinder, "now still
                           and silent" -- drawn dimmed/dead, same geometry
  f_string_dimension_housing_imploded
                          "imploded clump of metal" -- "An extremely dense
                           ball of imploded metal."  #, dark_gray
  f_utility_shelf_string_warped
                          "warped utility shelf" -- "cleaved into three
                           pieces, now mysteriously floating in space in a
                           stair-like pattern, disconnected entirely."
                           {, light_gray
  f_metal_crate_o_string_warped
                          "glowing metal crate" -- "its panels are
                           impossibly thin planes of magenta light."
                           X, magenta, light_emitted 40
  f_dead_exodii           "remains of a cyborg" -- "heavily rusted remains of
                           some sort of robotic knight... frozen on its knee,
                           head bowed, hands gripped around the barrel of a
                           gigantic rifle stuck into the dirt... Overgrowth
                           splits the gaps of its plating."  @, dark_gray
  f_dead_exodii_heartless "crumbled remains of a cyborg" -- "A pile of rusted
                           metal that resembles a collapsed statue."  @
  f_alien_weed_1          "a knee-high tightly-coiled bundle of reddish-yellow
                           stalks with jagged leaves extending every which
                           way."  color red/light_red
  f_alien_weed_2          "delicate bluish-white strands with small white
                           globes along their length... float almost like
                           seaweed."  color white/blue
  f_alien_weed_3          "a single, cylindrical dark-red trunk with a small
                           cluster of leaves on top."  color red

Design: UltiCa decomposed layout, all sprites 32x32 on the normal sheet,
furniture drawn as objects on a transparent background (the portal ring
and the imploded clump are solid structures; the floor rings lie flat).
Entries are {"id", "fg": id, "bg": ""} per this batch's spec. Flat fills,
dark outlines, one highlight pass -- UltiCa style, sharing the alien
meadow terrain palette (dark reds, waxy yellows, teal glows). Solid
surfaces are always fully opaque; translucency appears only where the
material itself is gas, glow, or light, and is built by compositing an
overlay (Image.alpha_composite), never by drawing semi-transparent
pixels onto a solid. Deterministic: every random choice comes from a
per-sprite seeded Random.
Output: pngs_normal_32x32/furniture/<id>/ (png + tile-entry json + credits).
"""
from PIL import Image, ImageDraw
import json, math, os, random

BASE = os.path.dirname(os.path.abspath(__file__))
S = 32

def new_img():
    return Image.new("RGBA", (S, S), (0, 0, 0, 0))

def overlay(img, draw_fn):
    """Composite a translucent-effects layer over img ('over' blending)."""
    layer = new_img()
    d = ImageDraw.Draw(layer)
    draw_fn(d)
    img.alpha_composite(layer)
    return img

OUTLINE = (36, 34, 32, 255)
METAL = (168, 170, 180, 255)
METAL_MID = (126, 128, 140, 255)
METAL_DARK = (82, 84, 96, 255)
METAL_LINE = (52, 54, 64, 255)

# ---------------------------------------------------------------- scarred lump
def draw_scar():
    rng = random.Random(4101)
    img = new_img(); d = ImageDraw.Draw(img)
    base = (66, 100, 46, 255)
    cols = [(84, 128, 58, 255), (100, 148, 68, 255), (74, 114, 52, 255),
            (116, 164, 78, 255)]
    # lumpy flesh mound: overlapping opaque ellipses, bottom-anchored
    for _ in range(11):
        cx = rng.uniform(7, 25); cy = rng.uniform(19, 27)
        r = rng.uniform(4.5, 8.5)
        d.ellipse([cx - r, cy - r * 0.62, cx + r, cy + r * 0.62], fill=base)
    for _ in range(46):
        cx = rng.uniform(4, 28); cy = rng.uniform(14, 30)
        r = rng.uniform(1.4, 3.2)
        d.ellipse([cx - r, cy - r * 0.7, cx + r, cy + r * 0.7],
                  fill=cols[rng.randrange(len(cols))])
    # scar gashes: jagged dark-red wounds with a paler rim
    for (x0, y0, x1, y1) in [(8, 20, 14, 24), (18, 17, 24, 21), (12, 25, 19, 27)]:
        pts = [(x0 + (x1 - x0) * t, y0 + (y1 - y0) * t + rng.uniform(-1, 1))
               for t in (0, 0.33, 0.66, 1.0)]
        d.line(pts, fill=(196, 120, 104, 255), width=2)
        d.line(pts, fill=(128, 44, 40, 255), width=1)
    # injured orifices: dark pits
    for (x, y, r) in [(11, 22, 1.9), (21, 24, 2.3), (16, 19, 1.5), (25, 19, 1.3)]:
        d.ellipse([x - r, y - r * 0.8, x + r, y + r * 0.8], fill=(26, 38, 22, 255))
        d.arc([x - r, y - r * 0.8, x + r, y + r * 0.8], 180, 330,
              fill=(140, 180, 104, 255), width=1)
    # belched gas: translucent pale-green puffs, composited (gas is not solid)
    def gas(dg):
        for (x, y, r) in [(10, 11, 3.4), (17, 8, 2.6), (23, 12, 3.0), (14, 13, 2.0)]:
            dg.ellipse([x - r, y - r * 0.7, x + r, y + r * 0.7],
                       fill=(178, 226, 150, 74))
        for _ in range(7):
            x = rng.uniform(7, 26); y = rng.uniform(4, 13)
            dg.point((x, y), fill=(200, 240, 170, 110))
    overlay(img, gas)
    return img

# ---------------------------------------------------------------- portal rings
def draw_portal_enclosure():
    rng = random.Random(4102)
    img = new_img(); d = ImageDraw.Draw(img)
    # dozens of wires trailing from the ring to the floor and sides
    for _ in range(14):
        sx = rng.uniform(6, 26); sy = rng.uniform(24, 30)
        ex = sx + rng.uniform(-9, 9); ey = 31
        col = (58, 60, 68, 255) if rng.random() < 0.7 else (146, 88, 50, 255)
        d.line([sx, sy, (sx + ex) / 2 + rng.uniform(-2, 2), (sy + ey) / 2, ex, ey],
               fill=col, width=1)
    # the heavy standing ring: thick band, dark outline, lit rim
    d.ellipse([3, 1, 29, 30], outline=METAL_LINE, width=8)
    d.ellipse([3, 1, 29, 30], outline=METAL, width=5)
    d.arc([3, 1, 29, 30], 150, 260, fill=(216, 218, 226, 255), width=2)
    d.arc([3, 1, 29, 30], 300, 60, fill=METAL_DARK, width=2)
    # bolt heads around the band
    for a in range(0, 360, 45):
        x = 16 + 11.5 * math.cos(math.radians(a)); y = 15.5 + 12.5 * math.sin(math.radians(a))
        d.ellipse([x - 1.1, y - 1.1, x + 1.1, y + 1.1], fill=METAL_DARK)
    # charred residue on the lower band + acrid olive streaks
    for _ in range(9):
        x = rng.uniform(8, 24); y = rng.uniform(24, 29)
        d.ellipse([x - 1.8, y - 1.0, x + 1.8, y + 1.0], fill=(30, 26, 22, 255))
    for _ in range(5):
        x = rng.uniform(9, 23); y = rng.uniform(22, 26)
        d.line([x, y, x + rng.uniform(-1, 1), y + rng.uniform(2, 4)],
               fill=(96, 90, 44, 255), width=1)
    return img

def draw_floor_ring():
    # One drawing for both floor-ring entries: their game data is verbatim
    # identical (name, description, symbol, color), like the string
    # dimension's t_string_xy / t_string_xy_plane_1.
    rng = random.Random(4103)
    img = new_img(); d = ImageDraw.Draw(img)
    # charred spot the ring surrounds
    d.ellipse([8, 13, 24, 22], fill=(26, 22, 19, 255))
    for _ in range(16):
        x = rng.uniform(9, 23); y = rng.uniform(14, 21)
        d.point((x, y), fill=(58, 50, 42, 255))
    for _ in range(6):  # acrid residue flecks
        x = rng.uniform(10, 22); y = rng.uniform(14, 21)
        d.point((x, y), fill=(110, 102, 48, 255))
    # the heavy ring lying flat: perspective ellipse band
    d.ellipse([2, 9, 30, 25], outline=METAL_LINE, width=5)
    d.ellipse([2, 9, 30, 25], outline=METAL, width=3)
    d.arc([2, 9, 30, 25], 170, 300, fill=(216, 218, 226, 255), width=1)
    # a few wires trailing away along the floor
    for (sx, ex) in [(4, 0), (28, 31), (7, 2)]:
        d.line([sx, 22, ex, 27 + rng.uniform(0, 3)], fill=(58, 60, 68, 255), width=1)
    return img

# ---------------------------------------------------------------- antenna
def draw_locator_antenna():
    rng = random.Random(4104)
    img = new_img(); d = ImageDraw.Draw(img)
    # tall pole, floor-to-roof
    d.rectangle([15, 1, 17, 29], fill=METAL_MID)
    d.line([15, 1, 15, 29], fill=(206, 208, 216, 255), width=1)
    d.line([17, 1, 17, 29], fill=METAL_LINE, width=1)
    # tip: small crossbar + knob
    d.line([11, 3, 21, 3], fill=METAL_DARK, width=2)
    d.ellipse([14.8, 0, 17.2, 2.2], fill=METAL)
    # the small set of holes near the base
    for y in (22, 25, 28):
        d.ellipse([15.2, y - 0.9, 16.8, y + 0.9], fill=(18, 18, 22, 255))
    # mounting base
    d.rectangle([11, 29, 21, 31], fill=METAL_DARK)
    d.line([11, 29, 21, 29], fill=METAL, width=1)
    return img

# ---------------------------------------------------------------- crystal flower
def draw_crystalline_flower():
    rng = random.Random(4105)
    img = new_img(); d = ImageDraw.Draw(img)
    pale = (168, 240, 190, 255); mid = (120, 216, 156, 255); deep = (84, 186, 124, 255)
    # crystalline stem: stacked jagged shards
    for (x0, y0, x1, y1) in [(15, 30, 17, 24), (14.6, 25, 17.4, 20), (15.2, 21, 16.8, 17)]:
        d.polygon([(x0, y0), (x1, y0 - 1), (x1 - 0.4, y1), (x0 + 0.4, y1)], fill=mid)
        d.line([(x0 + 0.7, y0), (x0 + 0.7, y1)], fill=pale, width=1)
    # petal crystals radiating from the head at (16, 14)
    for a in (-70, -35, 0, 35, 70, 180 - 35, 180 + 35):
        rad = math.radians(a - 90)
        ln = 6.5 if abs(a) < 80 else 4.5
        ex, ey = 16 + math.cos(rad) * ln, 14 + math.sin(rad) * ln * 0.8
        bx, by = 16 - math.sin(rad) * 1.6, 14 + math.cos(rad) * 1.6
        b2x, b2y = 16 + math.sin(rad) * 1.6, 14 - math.cos(rad) * 1.6
        d.polygon([(bx, by), (ex, ey), (b2x, b2y)], fill=pale)
        d.line([(bx, by), (ex, ey)], fill=(240, 255, 246, 255), width=1)
    d.polygon([(13.8, 16), (16, 11.4), (18.2, 16), (16, 17.6)], fill=deep)
    d.point((16, 13.6), fill=(255, 255, 255, 255))
    # fallen shards at the base + sparkles
    d.polygon([(8, 30), (10.4, 27.6), (11, 30)], fill=mid)
    d.polygon([(21, 30), (22.4, 28.2), (23.4, 30)], fill=pale)
    for (x, y) in [(9, 12), (24, 9), (20, 19)]:
        d.point((x, y), fill=(255, 255, 255, 255))
    return img

# ---------------------------------------------------------------- impaled
def _crystal(d, dark=False):
    """The glowing ground crystal: a jagged shard, base to tip."""
    if dark:
        body, facet, edge = (26, 28, 34, 255), (46, 50, 60, 255), (96, 104, 120, 255)
    else:
        body, facet, edge = (188, 238, 222, 255), (148, 222, 200, 255), (246, 255, 250, 255)
    d.polygon([(9, 31), (13, 12), (16, 1), (19, 10), (23, 31)], fill=body)
    d.polygon([(13, 12), (16, 1), (16.8, 20), (14, 26)], fill=facet)
    d.polygon([(19, 10), (23, 31), (18, 28)], fill=facet)
    d.line([(13, 12), (16, 1), (19, 10)], fill=edge, width=1)
    d.line([(16, 1), (16.8, 20)], fill=edge, width=1)
    # side shards seeping out of the ground
    d.polygon([(5, 31), (8, 22), (10.5, 31)], fill=body)
    d.polygon([(22, 31), (25, 24), (27, 31)], fill=facet)

def _impaled(armor, armor_dark, accent, helmet):
    rng = random.Random(4106)
    img = new_img()
    # glow halo behind the crystal (light, not a solid surface)
    overlay(img, lambda dg: dg.ellipse([4, 0, 28, 31], fill=(150, 255, 214, 42)))
    d = ImageDraw.Draw(img)
    _crystal(d)
    # refraction ghost: the body's edge duplicated, offset, in pale cyan
    def ghost(dg):
        dg.polygon([(11, 9), (21, 9), (22.5, 18), (10, 18)], fill=(190, 255, 240, 80))
    overlay(img, ghost)
    # the hanging body, pierced at the torso, limbs limp
    d.polygon([(11.5, 9), (20.5, 9), (21.5, 17), (10.5, 17)], fill=armor)  # torso
    d.line([(11.5, 12.5), (21.5, 12.5)], fill=accent, width=1)              # chest mark
    d.ellipse([9.6, 7.6, 13.4, 11.4], fill=armor_dark)                     # pauldron L
    d.ellipse([18.6, 7.6, 22.4, 11.4], fill=armor_dark)                    # pauldron R
    d.line([(10.5, 11), (8.6, 19)], fill=armor_dark, width=3)              # arm L hangs
    d.line([(21.5, 11), (23.4, 19)], fill=armor_dark, width=3)             # arm R hangs
    d.line([(13, 17), (12, 24)], fill=armor_dark, width=3)                 # leg L dangles
    d.line([(19, 17), (20.4, 23)], fill=armor_dark, width=3)               # leg R dangles
    # head bowed forward over the crystal tip
    d.ellipse([13.6, 3.4, 18.4, 8.2], fill=helmet)
    d.arc([13.6, 3.4, 18.4, 8.2], 20, 160, fill=accent, width=1)
    d.line([(14.6, 7.2), (17.4, 7.2)], fill=(16, 16, 20, 255), width=1)   # visor slit
    return img

def draw_impaled_officer():
    # high-tech armor: clean grays, red accents (game color red)
    return _impaled((128, 132, 144, 255), (88, 92, 104, 255),
                    (186, 44, 44, 255), (104, 108, 120, 255))

def draw_impaled_soldier():
    # military armor: browns and olive (game color brown)
    return _impaled((112, 90, 60, 255), (82, 64, 44, 255),
                    (70, 76, 52, 255), (96, 76, 52, 255))

def draw_empty_spike():
    rng = random.Random(4107)
    img = new_img(); d = ImageDraw.Draw(img)
    _crystal(d, dark=True)
    # strange refraction: thin spectral streaks along the facets
    for pts, col in [([(13.5, 11), (16.2, 2.5)], (120, 224, 255, 255)),
                     ([(16.8, 19), (19, 11)], (255, 130, 224, 255)),
                     ([(14.2, 25), (16.6, 20)], (255, 240, 150, 255)),
                     ([(18.5, 27), (22, 31)], (120, 224, 255, 255))]:
        d.line(pts, fill=col, width=1)
    d.point((16, 1), fill=(255, 255, 255, 255))
    return img

# ---------------------------------------------------------------- pylon
def _pylon(dim):
    rng = random.Random(4108)
    img = new_img(); d = ImageDraw.Draw(img)
    def c(rgb):
        if dim:
            g = int(0.45 * (0.35 * rgb[0] + 0.45 * rgb[1] + 0.2 * rgb[2]))
            return (g + 14, g + 10, g + 6, 255)
        return rgb + (255,)
    # floor-to-ceiling cylinder
    d.rectangle([9, 0, 23, 31], fill=c((122, 64, 36)))
    d.rectangle([11, 0, 14, 31], fill=c((198, 112, 62)))   # lit face
    d.rectangle([20, 0, 23, 31], fill=c((84, 44, 26)))      # shadow face
    # tight copper windings: a line every 2 px, alternating tone
    for y in range(1, 31, 2):
        d.line([9, y, 23, y], fill=c((156, 84, 44)) if (y // 2) % 2 else c((104, 54, 30)),
               width=1)
    d.line([11, 0, 11, 31], fill=c((226, 140, 80)), width=1)
    # cables connected to the base
    for (x0, x1) in [(10, 2), (13, 7), (20, 27), (22, 31)]:
        pts = [(x0, 30), ((x0 + x1) / 2, 29 + rng.uniform(0, 1.4)), (x1, 31)]
        d.line(pts, fill=(40, 38, 40, 255), width=2)
    if dim:
        # one cable severed, frayed end curled beside the base
        d.line([18, 29, 21, 27.4, 23.4, 28.2], fill=(40, 38, 40, 255), width=1)
        d.point((23.8, 28.4), fill=(150, 90, 50, 255))
    else:
        # the hum: a warm working glow at the core + faint vibration ghosts
        overlay(img, lambda dg: dg.rectangle([12, 6, 20, 26], fill=(255, 176, 84, 38)))
        def ghosts(dg):
            for x in (6.5, 25.5):
                dg.line([x, 6, x, 25], fill=(255, 200, 120, 60), width=1)
        overlay(img, ghosts)
    return img

def draw_pylon():
    return _pylon(False)

def draw_pylon_inert():
    return _pylon(True)

# ---------------------------------------------------------------- imploded clump
def draw_housing_imploded():
    rng = random.Random(4111)
    img = new_img(); d = ImageDraw.Draw(img)
    # ground shadow
    d.ellipse([4, 26, 28, 31], fill=(24, 24, 28, 255))
    # the dense ball: an irregular crumpled sphere
    d.ellipse([5, 6, 27, 28], fill=(88, 90, 98, 255))
    facet_cols = [(64, 66, 74, 255), (104, 106, 114, 255), (52, 54, 62, 255),
                  (120, 122, 130, 255)]
    for _ in range(26):  # crushed facets
        x = rng.uniform(7, 25); y = rng.uniform(8, 26)
        w = rng.uniform(2.4, 5.4); h = rng.uniform(1.8, 4.2)
        d.polygon([(x, y), (x + w, y + rng.uniform(-1, 1)),
                   (x + w * 0.6, y + h), (x - w * 0.2, y + h * 0.7)],
                  fill=facet_cols[rng.randrange(len(facet_cols))])
    for _ in range(7):  # inward dents: dark arcs
        x = rng.uniform(8, 22); y = rng.uniform(9, 24); r = rng.uniform(1.6, 3.2)
        d.arc([x - r, y - r, x + r, y + r], rng.uniform(0, 180), rng.uniform(190, 340),
              fill=(30, 31, 36, 255), width=1)
    # twisted shards sticking out of the mass
    for (x, y, ex, ey) in [(8, 10, 4.6, 6.4), (24, 12, 28.4, 9.4), (20, 7.4, 22.6, 3.8)]:
        d.polygon([(x, y), (ex, ey), (x + 2.4, y + 1.6)], fill=(70, 72, 80, 255))
    d.arc([5, 6, 27, 28], 150, 250, fill=(152, 154, 162, 255), width=1)  # one lit rim
    return img

# ---------------------------------------------------------------- warped shelf
def _shelf_fragment(d, x, y, rng):
    """One cleaved shelf section: two uprights, two boards, ~11x9 px."""
    frame = (96, 100, 110, 255); board = (172, 180, 190, 255); board_hi = (206, 214, 222, 255)
    d.line([x, y + 1, x, y + 9], fill=frame, width=2)
    d.line([x + 10, y, x + 10, y + 8], fill=frame, width=2)
    for dy in (1.5, 6.5):
        d.rectangle([x - 0.6, y + dy, x + 10.6, y + dy + 1.8], fill=board)
        d.line([x - 0.6, y + dy, x + 10.6, y + dy], fill=board_hi, width=1)
    # a warped sag in the top board
    d.line([x + 2, y + 2.6, x + 5, y + 3.4, x + 8, y + 2.6], fill=frame, width=1)

def draw_warped_shelf():
    rng = random.Random(4112)
    img = new_img(); d = ImageDraw.Draw(img)
    # three pieces floating in a stair-like pattern, disconnected
    _shelf_fragment(d, 2, 21, rng)    # low
    _shelf_fragment(d, 11, 12, rng)   # middle, floating
    _shelf_fragment(d, 20, 3, rng)    # high
    return img

# ---------------------------------------------------------------- glowing crate
def draw_glowing_crate():
    rng = random.Random(4113)
    img = new_img()
    # the panels are planes of magenta light: translucent faces, composited
    def faces(dg):
        dg.polygon([(7, 11), (22, 11), (22, 28), (7, 28)], fill=(255, 60, 255, 62))
        dg.polygon([(7, 11), (12, 5), (27, 5), (22, 11)], fill=(255, 60, 255, 44))
        dg.polygon([(22, 11), (27, 5), (27, 22), (22, 28)], fill=(255, 60, 255, 30))
        dg.rectangle([10, 15, 19, 24], fill=(255, 140, 255, 26))
    overlay(img, faces)
    d = ImageDraw.Draw(img)
    edge = (255, 96, 255, 255); node = (255, 196, 255, 255)
    # impossibly thin bright edges
    for pts in [[(7, 11), (22, 11), (22, 28), (7, 28), (7, 11)],
                [(7, 11), (12, 5), (27, 5), (22, 11)],
                [(22, 11), (27, 5), (27, 22), (22, 28)],
                [(7, 28), (12, 22), (27, 22)], [(12, 5), (12, 22)]]:
        d.line(pts, fill=edge, width=1)
    for (x, y) in [(7, 11), (22, 11), (22, 28), (7, 28), (12, 5), (27, 5), (27, 22), (12, 22)]:
        d.point((x, y), fill=node)
    return img

# ---------------------------------------------------------------- dead exodii
RUST = [(96, 88, 84, 255), (74, 66, 62, 255), (110, 100, 92, 255), (58, 52, 50, 255)]
RUST_FLECK = (156, 88, 46, 255)

def draw_dead_exodii():
    rng = random.Random(4114)
    img = new_img(); d = ImageDraw.Draw(img)
    # dirt the rifle is stuck into
    d.ellipse([17, 27, 30, 31], fill=(92, 72, 50, 255))
    d.ellipse([19, 27.6, 28, 30], fill=(112, 90, 62, 255))
    # the gigantic rifle: barrel angled down into the dirt
    d.line([(19.5, 5), (25.5, 28)], fill=(46, 44, 44, 255), width=3)
    d.line([(19.5, 5), (25.5, 28)], fill=(84, 80, 78, 255), width=1)
    d.polygon([(17.6, 3), (21.4, 3.8), (20.6, 7), (18.2, 6.4)], fill=RUST[3])  # receiver
    # the kneeling knight, leaning on the rifle: torso bowed forward
    d.polygon([(8, 12), (17, 10), (19.5, 19), (10, 21)], fill=RUST[0])
    d.polygon([(8, 12), (17, 10), (17.8, 13.4), (9, 15)], fill=RUST[2])
    d.ellipse([5.6, 8.6, 11.4, 13.4], fill=RUST[1])   # pauldron L
    d.ellipse([15.4, 7.8, 21, 12.4], fill=RUST[1])    # pauldron R
    # head bowed low between the shoulders
    d.ellipse([11.4, 6.2, 16.6, 11], fill=RUST[3])
    d.arc([11.4, 6.2, 16.6, 11], 300, 130, fill=RUST[2], width=1)
    d.line([(12.6, 9.6), (15.4, 9.6)], fill=(14, 13, 13, 255), width=1)  # dark visor
    # arms reaching down-forward, hands gripped around the barrel
    d.line([(9, 13), (16, 17.4), (19.8, 12.6)], fill=RUST[1], width=3)
    d.line([(18, 12), (20.2, 13.2)], fill=RUST[1], width=3)
    d.ellipse([18.2, 11.2, 21.6, 14.2], fill=RUST[3])  # gripping hands
    # kneeling legs: one knee grounded, the other folded
    d.ellipse([6.4, 24.4, 12.4, 28.6], fill=RUST[1])
    d.polygon([(13, 20), (18, 21), (17, 27), (12.4, 26)], fill=RUST[0])
    # rust flecks + overgrowth splitting the plating gaps
    for _ in range(14):
        x = rng.uniform(6, 21); y = rng.uniform(7, 27)
        d.point((x, y), fill=RUST_FLECK)
    for (x0, y0, x1, y1) in [(9, 15, 7.6, 22), (16.6, 14, 18.6, 20), (12, 20.4, 11, 25)]:
        d.line([x0, y0, (x0 + x1) / 2 + 1, (y0 + y1) / 2, x1, y1],
               fill=(84, 124, 60, 255), width=1)
        d.point(((x0 + x1) / 2 + 1.6, (y0 + y1) / 2), fill=(110, 156, 76, 255))
    return img

def draw_dead_exodii_heartless():
    rng = random.Random(4115)
    img = new_img(); d = ImageDraw.Draw(img)
    d.ellipse([3, 27, 29, 31], fill=(26, 25, 24, 255))  # shadow
    # a collapsed heap of rusted plates, like a fallen statue
    heap = [(5, 24, 16, 29), (13, 22, 25, 28), (8, 19, 20, 25), (15, 17, 24, 23),
            (10, 15, 19, 20)]
    for i, (x0, y0, x1, y1) in enumerate(heap):
        d.rounded_rectangle([x0, y0, x1, y1], radius=2, fill=RUST[i % 3])
        d.line([x0 + 1, y0 + 1, x1 - 1, y0 + 1], fill=RUST[2], width=1)
    # a cracked helm dome half-buried on top
    d.pieslice([12.4, 10.4, 20.4, 17.4], 180, 360, fill=RUST[3])
    d.line([(13.4, 13.8), (19.4, 13.8)], fill=(14, 13, 13, 255), width=1)
    d.line([(16.4, 10.8), (15.6, 13.6)], fill=(20, 18, 18, 255), width=1)  # the crack
    # scattered shards trailing away
    for (x, y, w) in [(3, 28, 3), (26, 27, 2.4), (24, 29.4, 3.4), (6, 30, 2)]:
        d.polygon([(x, y), (x + w, y - 1), (x + w + 0.6, y + 1)], fill=RUST[1])
    for _ in range(18):
        x = rng.uniform(4, 27); y = rng.uniform(14, 29)
        d.point((x, y), fill=RUST_FLECK)
    for _ in range(5):  # sparse moss: even the overgrowth is going
        x = rng.uniform(6, 24); y = rng.uniform(18, 27)
        d.point((x, y), fill=(84, 124, 60, 255))
    return img

# ---------------------------------------------------------------- alien weeds
MEADOW_RED = (140, 30, 34, 255)
MEADOW_RED_DARK = (104, 20, 26, 255)
MEADOW_YELLOW = (232, 196, 64, 255)

def draw_weed_1():
    rng = random.Random(4116)
    img = new_img(); d = ImageDraw.Draw(img)
    # tightly coiled stalks: concentric spiral arcs at the base
    for r, col in [(9.5, MEADOW_RED_DARK), (7.4, MEADOW_RED), (5.2, (168, 52, 44, 255)),
                   (3.2, MEADOW_YELLOW)]:
        d.arc([16 - r, 30 - r * 0.9, 16 + r, 30 + r * 0.28], 180, 360, fill=col, width=2)
    # stalks rising out of the coil
    for x in (9, 13, 19, 23):
        d.line([x, 29, x + rng.uniform(-1.6, 1.6), rng.uniform(15, 19)],
               fill=MEADOW_RED, width=2)
    # jagged leaves extending every which way
    for _ in range(11):
        cx = rng.uniform(6, 26); cy = rng.uniform(12, 24)
        a = rng.uniform(0, math.tau); ln = rng.uniform(3.4, 5.6)
        ex, ey = cx + math.cos(a) * ln, cy + math.sin(a) * ln * 0.7
        mx, my = cx + math.cos(a + 0.5) * ln * 0.5, cy + math.sin(a + 0.5) * ln * 0.35
        col = MEADOW_RED if rng.random() < 0.6 else MEADOW_YELLOW
        d.polygon([(cx, cy), (mx, my), (ex, ey), (mx - (my - cy) * 0.4, my + (mx - cx) * 0.4)],
                  fill=col)
        d.line([(cx, cy), (ex, ey)], fill=MEADOW_RED_DARK, width=1)
    return img

def draw_weed_2():
    rng = random.Random(4117)
    img = new_img(); d = ImageDraw.Draw(img)
    strands = [(7, 0.0), (11.5, 1.4), (16, 2.8), (20.5, 4.2), (25, 5.6), (14, 7.0), (23, 8.4)]
    for (x0, ph) in strands:
        pts = [(x0 + 2.2 * math.sin(y / 6.5 + ph), y) for y in range(9, 32, 2)]
        col = (196, 220, 246, 255) if rng.random() < 0.5 else (240, 246, 252, 255)
        d.line(pts, fill=col, width=1)
        # small white globes along the strand's length
        for t in (0.22, 0.48, 0.74):
            gx, gy = pts[int(t * (len(pts) - 1))]
            d.ellipse([gx - 1.3, gy - 1.3, gx + 1.3, gy + 1.3], fill=(250, 250, 253, 255))
            d.point((gx - 0.4, gy - 0.4), fill=(178, 205, 235, 255))
    # drifting tips: the strands float like seaweed, tips curling free
    for (x0, ph) in strands[:4]:
        ex = x0 + 2.2 * math.sin(9 / 6.5 + ph)
        d.arc([ex - 2.4, 4.6, ex + 2.4, 9.6], 200, 330,
              fill=(240, 246, 252, 255), width=1)
    return img

def draw_weed_3():
    rng = random.Random(4118)
    img = new_img(); d = ImageDraw.Draw(img)
    # a single cylindrical dark-red trunk, faintly banded (hollow rings)
    d.rectangle([13, 11, 19, 31], fill=MEADOW_RED_DARK)
    d.line([14.4, 11, 14.4, 31], fill=(168, 52, 44, 255), width=1)
    d.line([18, 11, 18, 31], fill=(72, 14, 18, 255), width=1)
    for y in (16, 21, 26):
        d.line([13, y, 19, y], fill=(84, 18, 22, 255), width=1)
    d.ellipse([13, 9.6, 19, 12.2], fill=(88, 20, 24, 255))  # the hollow mouth
    d.ellipse([14.6, 10.2, 17.4, 11.6], fill=(30, 8, 10, 255))
    # a small cluster of leaves on top
    for (cx, cy, col) in [(10.6, 8, MEADOW_RED), (16, 5.6, (172, 44, 44, 255)),
                          (21.4, 8, MEADOW_RED), (13, 7.4, (196, 60, 52, 255)),
                          (19, 7.2, MEADOW_RED_DARK)]:
        d.ellipse([cx - 2.6, cy - 1.6, cx + 2.6, cy + 1.6], fill=col)
        d.line([(16, 10.6), (cx, cy + 1)], fill=MEADOW_RED_DARK, width=1)
    return img

# ---------------------------------------------------------------- jobs
JOBS = [
    ("f_alien_scar", draw_scar()),
    ("f_exodii_portal_enclosure", draw_portal_enclosure()),
    ("f_exodii_floor_portal_enclosure", draw_floor_ring()),
    ("f_string_dimension_floor_portal_enclosure", draw_floor_ring()),
    ("f_exodii_locator_antenna", draw_locator_antenna()),
    ("f_crystalline_flower", draw_crystalline_flower()),
    ("f_impaled_officer", draw_impaled_officer()),
    ("f_impaled_soldier", draw_impaled_soldier()),
    ("f_empty_spike", draw_empty_spike()),
    ("f_string_dimension_pylon", draw_pylon()),
    ("f_string_dimension_pylon_inert", draw_pylon_inert()),
    ("f_string_dimension_housing_imploded", draw_housing_imploded()),
    ("f_utility_shelf_string_warped", draw_warped_shelf()),
    ("f_metal_crate_o_string_warped", draw_glowing_crate()),
    ("f_dead_exodii", draw_dead_exodii()),
    ("f_dead_exodii_heartless", draw_dead_exodii_heartless()),
    ("f_alien_weed_1", draw_weed_1()),
    ("f_alien_weed_2", draw_weed_2()),
    ("f_alien_weed_3", draw_weed_3()),
]

CREDITS = ("Drawn from the terrain's in-game description for the CDDA Voxel\n"
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.\n")

for tid, img in JOBS:
    folder = os.path.join(BASE, "pngs_normal_32x32", "furniture", tid)
    os.makedirs(folder, exist_ok=True)
    img.save(os.path.join(folder, tid + ".png"))
    entry = {"id": tid, "fg": tid, "bg": ""}
    with open(os.path.join(folder, tid + ".json"), "w") as f:
        json.dump([entry], f, indent=2)
        f.write("\n")
    with open(os.path.join(folder, "credits.txt"), "w") as f:
        f.write(CREDITS)
    print("wrote", tid)

# ---------------------------------------------------------------- preview
def tile_img(cat, tid):
    p = os.path.join(BASE, "pngs_normal_32x32", cat, tid, tid + ".png")
    return Image.open(p).convert("RGBA")

grass = tile_img("terrain", "t_alien_meadow_grass")

CELL = 64
COLS = 5
ROWS = (len(JOBS) + COLS - 1) // COLS
sheet = Image.new("RGBA", (COLS * CELL, ROWS * CELL), (24, 24, 28, 255))
for i, (tid, _img) in enumerate(JOBS):
    cell = grass.resize((CELL, CELL), Image.NEAREST)
    fg = tile_img("furniture", tid).resize((CELL, CELL), Image.NEAREST)
    cell.alpha_composite(fg)
    sheet.alpha_composite(cell, ((i % COLS) * CELL, (i // COLS) * CELL))
sheet.save(os.path.join(BASE, "preview_alien_furniture.png"))
print("wrote preview_alien_furniture.png", sheet.size)
