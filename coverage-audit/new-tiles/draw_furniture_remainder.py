#!/usr/bin/env python3
"""
Draw the missing furniture-remainder sprites from their game descriptions.

Phase 2g.4 furniture wave 2, the finisher batch: every furniture id in
coverage-audit/missing.json (missing_shipped) not covered by the named
parallel batches (furniture-terrains / furniture_vitrified /
furniture-alien / furniture_alien_meadow / furniture-regional-pseudo /
furniture-flora_marine / furniture-decorative) and not already foldered.

Sources (fork-src/cdda/data/json, CDDA 6a3e59e0):

  fireplaces:  f_hobo_stove "small improvised wood stove, made from a
               metal can" (red); f_solar_cooker "faceted reflector dish
               on a tripod" (yellow).
  medical:     f_exam_chair "medical exam chair with an overhead light,
               bolted to the floor"; f_surgical_table "padded surgical
               table mounted to a sturdy base" (named lab workbench).
  storage:     f_butter_churn "metal tube with a built-in mixer...
               pedal-driven" (light_cyan); f_metal_crate_military
               "large military container... lid bolted to the base and
               sealed" (light_gray); f_metal_crate_military_o "base
               half... inside has brackets for bolting engine on";
               f_torah_ark "ornately-carved cabinet... decorated with
               a tree motif" (i_brown).
  triffid:     f_brambles "a wall of tangled roots"; f_glowingbulb
               "green, onion-like bulb illuminating the area"
               (light_emitted 15); f_midden "a hole dug into the roots...
               a pretty unpleasant smell emits from it".
  generators:  all copy-from f_active_backup_generator, looks_like
               t_plut_generator, symbol 0, color green_white:
               f_12000_gasoline_generator "big and powerful, yet
               portable"; f_diesel_generator "big and powerful, used
               for street works"; f_military_generator "painted in
               camo"; f_portable_generator "small... people buy for a
               camp" -- one body plan, four paints/scales.
  misc:        f_portal_dungeon_reward_increaser "metanoiac possibility:
               something is there. It is what you could have been"
               (dark_gray); _inert "something you touched" (black).
  netherum overgrown: f_nl_climbing_vine "floor-to-ceiling vines,
               twisted and gnarled, like fraying wires, plastic sheen"
               (red); f_nl_pipe_plant "reeds almost identical in shape
               to plumbing pipes, wood-like material" (brown);
               f_nl_powerbox "rusted metal utility box, painted dark
               green, two rectangular pieces welded in the center";
               f_nl_powerbox_o "opened up. The interior is oily, fleshy
               meat, with interspersed wires... a rectangular
               depression with connections".
  netherum safehouse: f_nl_autodoc "lanky robotic limbs armed with
               surgical devices... barrel-like base on small casters...
               scalpel stropped against another arm... circular saw"
               (light_cyan); f_nl_autodoc_couch "hard metal table on
               wheels... screens dangle from the edge, black with
               burned-in lines"; f_nl_boiler "massive cylindrical
               device, exterior deeply clouded and yellowed... behind
               the plastic a large bulbous object, bolted down with
               thick hardware" (white); f_nl_safehouse_boiler "huge
               pot-bellied device between a coal stove and a boiler,
               hinged cage over the combustion chamber, three unmarked
               gauges, side crank, coiled copper hose" (red).
  obsolete:    f_mutant_tree_sprout "a tiny little sprout";
               f_mutant_tree_sapling "twisted and gnarled little tree";
               f_mutant_tree_adult "spreads its twisted branches at odd
               angles... almost a human figure with arms raised".
  tail:        f_solar_unit_v2 "mounted monocrystalline solar panel"
               (light_blue); f_fog_breather "a misty cloud of water
               vapor" (pink); f_clear / f_no_item "seeing this is a
               bug" debug placeholders (black, symbol #);
               f_string_dimension_dead_flower "glistening spindles with
               black, frostbitten petals from cracks in rocky ground";
               f_flower_marloss "bulb a brilliant cyan color" (cyan);
               f_grave_monument "obelisk: magnificent carved statue with
               an engraved plaque" (black_white); f_metal_detector
               "metal detector like at the airport" (light_blue);
               f_xedra_mobile_weather_station "mundane and otherworldly
               devices deployed from a sleek black case... telescoping
               stand" (dark_gray); f_sink_gravity "water basin fed by a
               tank suspended above, hand pump" (white); f_ball_mach
               "machine for launching sports balls, pair of motorized
               wheels" (dark_gray); f_roulette_table "huge table, grid
               painted onto the felt top, concave spinning wheel"
               (green); f_barber_chair "chair of cloth and steel,
               adjustable, swings" (dark_gray); f_beanbag "large cushion
               filled with beads"; f_bed_down "down mattress on a sturdy
               wooden frame" (magenta); f_bunkbed_down "two down
               mattresses, wooden frame"; f_firefly_terrarium "vivarium
               with soil, strange plants, a giant mutant firefly"
               (light_green, light_emitted 25); f_screw_press "large
               screw press for pressing oil from seed paste" (light_red);
               f_m224_mortar "deployed 60mm mortar, firing tables
               clipped to it" (dark_gray); f_nm_nether_lichen "bed of
               orange flowering lichen on yellow ichor" (green);
               f_sunkenhelicopter_seat_rotating "waterlogged upholstered
               bucket seat, degraded" (green).

Design: UltiCa decomposed layout, 32x32 normal sheet. Furniture is an
object on a transparent background (bg "" in the entry), unless the
piece is a solid structure filling the tile. Flat fills, dark
outlines, one highlight pass. Deterministic: per-sprite seeded Random.
This script ONLY writes its own batch's folders.
Output: pngs_normal_32x32/furniture/<id>/ (png + tile-entry json +
credits) and preview_furniture_remainder.png.
"""
from PIL import Image, ImageDraw
import json, math, os, random

BASE = os.path.dirname(os.path.abspath(__file__))
S = 32
OL = (26, 22, 20, 255)  # outline

def new_img():
    return Image.new("RGBA", (S, S), (0, 0, 0, 0))

def shade(c, f):
    return (max(0, min(255, int(c[0] * f))), max(0, min(255, int(c[1] * f))),
            max(0, min(255, int(c[2] * f))), c[3] if len(c) > 3 else 255)

def rect(d, box, fill, outline=OL, width=1):
    d.rectangle(box, fill=fill, outline=outline, width=width)

def ell(d, box, fill, outline=None):
    d.ellipse(box, fill=fill, outline=outline)

# ================================================================ fireplaces
def draw_hobo_stove():
    rng = random.Random(4101)
    img = new_img(); d = ImageDraw.Draw(img)
    rust, rust_d, metal = (146, 62, 40, 255), (104, 42, 28, 255), (120, 116, 110, 255)
    # stubby chimney pipe
    rect(d, [14, 3, 18, 9], metal)
    d.line([14, 5, 18, 5], fill=shade(metal, 0.7))
    # can body
    rect(d, [8, 9, 24, 27], rust)
    d.line([10, 9, 10, 27], fill=shade(rust, 1.25), width=1)  # highlight
    for y in (13, 22):  # can ridges
        d.line([8, y, 24, y], fill=rust_d)
    # fire door: dark opening with ember glow
    rect(d, [12, 16, 20, 24], (24, 14, 10, 255))
    for _ in range(6):
        x = rng.uniform(13, 19); y = rng.uniform(19, 23)
        d.point((x, y), fill=(255, rng.choice([120, 160, 190]), 40, 255))
    d.line([12, 16, 20, 16], fill=(70, 60, 55, 255))
    # little legs
    d.line([10, 27, 9, 30], fill=rust_d, width=2)
    d.line([22, 27, 23, 30], fill=rust_d, width=2)
    return img

def draw_solar_cooker():
    rng = random.Random(4102)
    img = new_img(); d = ImageDraw.Draw(img)
    steel = (96, 96, 102, 255)
    # tripod legs
    for (x0, x1) in [(16, 6), (16, 16), (16, 26)]:
        d.line([x0, 17, x1, 30], fill=steel, width=2)
    # faceted parabolic dish: concentric facet rings, silver-yellow
    cx, cy = 16, 11
    rings = [(11, (206, 200, 170, 255)), (8.5, (226, 220, 188, 255)),
             (6, (240, 234, 200, 255)), (3.5, (250, 246, 214, 255))]
    for r, col in rings:
        ell(d, [cx - r, cy - r * 0.62, cx + r, cy + r * 0.62], col, OL)
    for a in range(0, 360, 45):  # facet seams
        x = cx + 10.5 * math.cos(math.radians(a)); y = cy + 6.5 * math.sin(math.radians(a))
        d.line([cx, cy, x, y], fill=(178, 172, 146, 255))
    # focal arm + cooking point
    d.line([cx, cy + 4, cx, cy - 5], fill=steel, width=1)
    ell(d, [cx - 2, cy - 8, cx + 2, cy - 4], (60, 58, 54, 255), OL)
    d.point((cx, cy - 6), fill=(255, 240, 160, 255))
    return img

# ================================================================ medical
def draw_exam_chair():
    rng = random.Random(4103)
    img = new_img(); d = ImageDraw.Draw(img)
    vinyl, vinyl_d, steel = (64, 128, 122, 255), (46, 96, 92, 255), (150, 152, 156, 255)
    # overhead light: arm from behind + lamp head
    d.line([25, 28, 25, 8, 19, 5], fill=steel, width=2)
    ell(d, [13, 2, 20, 7], (226, 228, 220, 255), OL)
    d.point((16, 5), fill=(255, 250, 200, 255))
    # bolted pedestal base
    rect(d, [11, 27, 22, 30], shade(steel, 0.75))
    rect(d, [15, 21, 18, 27], steel)
    for (x, y) in [(12, 28), (21, 28)]:
        d.point((x, y), fill=(40, 40, 44, 255))
    # reclined seat + backrest + leg rest
    d.polygon([(6, 20), (18, 20), (18, 23), (6, 23)], fill=vinyl, outline=OL)
    d.polygon([(17, 20), (24, 10), (27, 12), (20, 22)], fill=vinyl, outline=OL)
    d.polygon([(6, 21), (2, 25), (4, 27), (8, 23)], fill=vinyl_d, outline=OL)
    d.line([8, 21, 16, 21], fill=shade(vinyl, 1.3))
    return img

def draw_surgical_table():
    rng = random.Random(4104)
    img = new_img(); d = ImageDraw.Draw(img)
    pad, pad_d, steel = (168, 182, 190, 255), (128, 142, 152, 255), (140, 142, 146, 255)
    # sturdy base + column
    rect(d, [9, 27, 24, 30], shade(steel, 0.7))
    rect(d, [14, 19, 19, 27], steel)
    # padded top with section seams and side rail
    rect(d, [3, 13, 29, 18], pad)
    d.line([3, 15, 29, 15], fill=shade(pad, 1.12))
    for x in (11, 20):
        d.line([x, 13, x, 18], fill=pad_d)
    d.line([3, 19, 29, 19], fill=shade(steel, 1.15))  # rail
    d.line([6, 18, 6, 20], fill=steel); d.line([26, 18, 26, 20], fill=steel)
    return img

# ================================================================ storage
def draw_butter_churn():
    rng = random.Random(4105)
    img = new_img(); d = ImageDraw.Draw(img)
    tube, tube_d, wood = (150, 196, 204, 255), (104, 148, 158, 255), (122, 88, 54, 255)
    # pedal + frame at the base
    d.line([9, 30, 23, 30], fill=shade(wood, 0.8), width=2)
    d.polygon([(11, 28), (19, 28), (17, 30), (12, 30)], fill=wood, outline=OL)
    d.line([10, 12, 8, 28], fill=wood, width=2)
    d.line([22, 12, 24, 28], fill=wood, width=2)
    # metal tube body
    rect(d, [10, 10, 22, 24], tube)
    d.line([12, 10, 12, 24], fill=shade(tube, 1.18), width=1)
    d.line([10, 14, 22, 14], fill=tube_d)
    # lid + dasher rod + handle
    rect(d, [9, 8, 23, 10], tube_d)
    d.line([16, 8, 16, 3], fill=shade(tube_d, 0.8), width=2)
    d.line([12, 3, 20, 3], fill=wood, width=2)
    return img

def _crate_body(d, body, dark):
    rect(d, [3, 7, 29, 28], body)
    for x in (5, 26):  # reinforcing ribs
        d.line([x + 1, 7, x + 1, 28], fill=dark)
    for _ in range(4):  # stencil marks
        pass

def draw_metal_crate_military():
    rng = random.Random(4106)
    img = new_img(); d = ImageDraw.Draw(img)
    body, dark, light = (168, 170, 172, 255), (110, 112, 116, 255), (206, 208, 210, 255)
    _crate_body(d, body, dark)
    # bolted lid seam across the top third
    rect(d, [3, 7, 29, 12], shade(body, 1.06))
    d.line([3, 12, 29, 12], fill=dark)
    for x in range(5, 29, 4):  # bolts
        d.point((x, 9), fill=(70, 72, 76, 255))
        d.point((x, 26), fill=(70, 72, 76, 255))
    d.line([5, 15, 5, 25], fill=light)  # highlight pass
    rect(d, [12, 17, 20, 21], None, dark)  # stencil plate outline
    d.line([14, 19, 18, 19], fill=dark)
    return img

def draw_metal_crate_military_o():
    rng = random.Random(4107)
    img = new_img(); d = ImageDraw.Draw(img)
    body, dark = (158, 160, 162, 255), (100, 102, 106, 255)
    # open base: rim walls around a dark interior
    rect(d, [3, 9, 29, 28], body)
    rect(d, [6, 12, 26, 25], (74, 74, 80, 255))
    # engine mounting brackets inside
    for (x, y) in [(9, 15), (21, 15), (9, 21), (21, 21)]:
        rect(d, [x, y, x + 3, y + 3], (142, 144, 148, 255), (60, 60, 64, 255))
        d.point((x + 1, y + 1), fill=(30, 30, 34, 255))
    d.line([3, 9, 29, 9], fill=(196, 198, 200, 255))  # rim highlight
    for x in range(5, 29, 6):
        d.point((x, 26), fill=(70, 72, 76, 255))
    return img

def draw_torah_ark():
    rng = random.Random(4108)
    img = new_img(); d = ImageDraw.Draw(img)
    wood, wood_d, gold = (122, 78, 44, 255), (84, 52, 28, 255), (206, 168, 84, 255)
    # crown top + body
    d.polygon([(6, 6), (26, 6), (23, 2), (9, 2)], fill=wood_d, outline=OL)
    rect(d, [5, 6, 27, 29], wood)
    d.line([16, 6, 16, 29], fill=wood_d)  # double doors
    rect(d, [5, 6, 27, 29], None, OL)
    # carved tree motif (Proverbs 3:18): trunk + canopy on both doors
    for cx in (10.5, 21.5):
        d.line([cx, 24, cx, 17], fill=gold, width=1)
        ell(d, [cx - 4, 8, cx + 4, 16], None, gold)
        d.line([cx, 17, cx - 3, 13], fill=gold)
        d.line([cx, 17, cx + 3, 12], fill=gold)
    for x in (14, 18):  # door pulls
        d.point((x, 20), fill=gold)
    d.line([7, 8, 7, 27], fill=shade(wood, 1.3))  # highlight
    return img

# ================================================================ triffid
def draw_brambles():
    rng = random.Random(4109)
    img = new_img(); d = ImageDraw.Draw(img)
    cols = [(74, 52, 30, 255), (94, 66, 38, 255), (58, 42, 24, 255), (110, 80, 46, 255)]
    # a wall of tangled roots: crossing arcs everywhere
    for _ in range(16):
        x0 = rng.uniform(-4, 30); y0 = rng.uniform(0, 31)
        x1 = x0 + rng.uniform(-14, 14); y1 = y0 + rng.uniform(-16, 16)
        col = cols[rng.randrange(len(cols))]
        d.line([x0, y0, (x0 + x1) / 2 + rng.uniform(-4, 4),
                (y0 + y1) / 2 + rng.uniform(-4, 4), x1, y1], fill=col, width=2)
    for _ in range(10):  # thorn nubs
        x = rng.uniform(2, 29); y = rng.uniform(2, 29)
        d.line([x, y, x + 1.6, y - 1.6], fill=(140, 116, 76, 255))
    return img

def draw_glowingbulb():
    rng = random.Random(4110)
    img = new_img(); d = ImageDraw.Draw(img)
    # roots at the base
    for (x0, x1) in [(4, 14), (10, 18), (18, 28), (24, 15)]:
        d.line([x0, 30, x1, 26], fill=(84, 58, 32, 255), width=2)
    # stem up to the bulb
    d.line([16, 27, 15, 17], fill=(70, 110, 44, 255), width=2)
    d.ellipse([9, 22, 14, 26], fill=(70, 110, 44, 255), outline=OL)  # leaf
    # the green onion-like bulb, glowing (light_emitted 15)
    ell(d, [9, 5, 23, 18], (120, 220, 90, 255), OL)
    ell(d, [12, 7, 19, 13], (178, 246, 140, 255))
    for x in (12, 16, 20):  # onion stripes
        d.line([x, 6, x - 1, 17], fill=(86, 178, 62, 255))
    for _ in range(7):  # glow motes
        x = rng.uniform(4, 27); y = rng.uniform(2, 20)
        d.point((x, y), fill=(210, 255, 170, 255))
    return img

def draw_midden():
    rng = random.Random(4111)
    img = new_img(); d = ImageDraw.Draw(img)
    # root mound with a dug hole
    ell(d, [2, 12, 30, 30], (96, 68, 40, 255), OL)
    for _ in range(9):
        x0 = rng.uniform(2, 28); y0 = rng.uniform(14, 28)
        d.line([x0, y0, x0 + rng.uniform(-5, 5), y0 + rng.uniform(-3, 3)],
               fill=(70, 48, 26, 255), width=2)
    ell(d, [10, 16, 23, 26], (16, 10, 6, 255))  # the hole
    ell(d, [12, 18, 20, 23], (30, 20, 10, 255))
    # unpleasant smell: pale wavy wisps
    for (x, ph) in [(12, 0.0), (17, 1.6), (21, 3.1)]:
        pts = [(x + 1.6 * math.sin(y / 3.0 + ph), y) for y in range(4, 15, 2)]
        d.line(pts, fill=(168, 178, 130, 255))
    return img

# ================================================================ generators
def _generator(seed, body, dark, accent, big, camo=False):
    rng = random.Random(seed)
    img = new_img(); d = ImageDraw.Draw(img)
    if big:
        box = [4, 8, 28, 27]
    else:
        box = [8, 12, 24, 26]
    x0, y0, x1, y1 = box
    if not big:  # carry handle on the portable
        d.arc([12, 6, 20, 13], start=180, end=360, fill=dark, width=2)
    rect(d, box, body)
    if camo:
        for _ in range(9):
            x = rng.uniform(x0 + 1, x1 - 5); y = rng.uniform(y0 + 1, y1 - 4)
            col = rng.choice([(86, 96, 58, 255), (64, 70, 46, 255), (110, 104, 74, 255)])
            ell(d, [x, y, x + rng.uniform(3, 6), y + rng.uniform(2, 4)], col)
    # tubular frame
    for x in (x0, x1 - 1):
        d.line([x, y0 - 2, x, y1 + 2], fill=dark, width=2)
    d.line([x0, y1 + 2, x1, y1 + 2], fill=dark, width=2)
    d.line([x0, y0 - 2, x1, y0 - 2], fill=dark, width=2)
    # fuel tank cap + engine vents
    ell(d, [x0 + 3, y0 + 2, x0 + 7, y0 + 5], dark, OL)
    for i in range(3):
        d.line([x0 + 3, y0 + 9 + i * 2, x0 + 10, y0 + 9 + i * 2], fill=dark)
    # control panel with dial + outlets
    px = x1 - 9
    rect(d, [px, y0 + 3, x1 - 2, y0 + 12], shade(body, 1.18), dark)
    ell(d, [px + 1, y0 + 4, px + 5, y0 + 8], (238, 238, 232, 255), dark)
    d.point((px + 3, y0 + 6), fill=(200, 40, 40, 255))
    rect(d, [px + 1, y0 + 9, px + 3, y0 + 11], accent)
    rect(d, [px + 4, y0 + 9, px + 6, y0 + 11], (40, 40, 44, 255))
    d.line([x0 + 2, y0 + 1, x0 + 2, y1 - 2], fill=shade(body, 1.25))  # highlight
    return img

def draw_gen_12000():
    return _generator(4112, (88, 118, 74, 255), (44, 62, 36, 255), (220, 220, 210, 255), True)

def draw_gen_diesel():
    return _generator(4113, (150, 142, 62, 255), (84, 78, 34, 255), (220, 220, 210, 255), True)

def draw_gen_military():
    return _generator(4114, (96, 104, 66, 255), (50, 56, 34, 255), (200, 200, 190, 255), True, camo=True)

def draw_gen_portable():
    return _generator(4115, (96, 128, 80, 255), (48, 68, 40, 255), (226, 90, 60, 255), False)

# ================================================================ portal misc
def _portal(seed, base, inner, glow):
    rng = random.Random(seed)
    img = new_img(); d = ImageDraw.Draw(img)
    # a jagged shard of "something": a not-quite-person standing there
    d.polygon([(16, 2), (21, 8), (20, 16), (23, 29), (18, 29), (16, 20),
               (14, 29), (9, 29), (12, 16), (11, 8)], fill=base, outline=shade(base, 0.6))
    ell(d, [13.5, 5, 18.5, 10], shade(base, 1.25))  # head-ish
    if glow:  # what you could have been: a faint brighter self inside
        ell(d, [14.5, 6, 17.5, 9], inner)
        d.line([16, 11, 16, 20], fill=inner)
        for _ in range(6):
            x = rng.uniform(8, 24); y = rng.uniform(3, 27)
            d.point((x, y), fill=inner)
    else:  # inert: cracks, no light
        d.line([13, 8, 17, 14, 14, 20], fill=shade(base, 1.6))
        d.line([19, 12, 16, 18], fill=shade(base, 1.6))
    return img

def draw_portal_increaser():
    return _portal(4116, (74, 74, 80, 255), (168, 172, 190, 255), True)

def draw_portal_increaser_inert():
    return _portal(4117, (16, 16, 18, 255), (0, 0, 0, 255), False)

# ================================================================ netherum overgrown
def draw_nl_climbing_vine():
    rng = random.Random(4118)
    img = new_img(); d = ImageDraw.Draw(img)
    cols = [(150, 44, 40, 255), (178, 58, 50, 255), (120, 34, 32, 255)]
    # floor-to-ceiling gnarled vines, like fraying loose wires
    for x0 in (4, 9, 14, 19, 24, 28):
        pts = [(x0 + 2.2 * math.sin(y / 5.0 + x0), y) for y in range(0, 32, 2)]
        d.line(pts, fill=cols[x0 % 3], width=2)
        hi = [(x - 0.8, y) for x, y in pts]  # plastic sheen
        d.line(hi[::2], fill=(226, 130, 118, 255))
    for _ in range(9):  # frayed wire ends curling off
        x = rng.uniform(2, 28); y = rng.uniform(2, 28)
        d.arc([x, y, x + 5, y + 5], start=rng.uniform(0, 180),
              end=rng.uniform(200, 340), fill=cols[rng.randrange(3)])
    return img

def draw_nl_pipe_plant():
    rng = random.Random(4119)
    img = new_img(); d = ImageDraw.Draw(img)
    wood, wood_d, wood_l = (140, 104, 62, 255), (96, 68, 40, 255), (178, 140, 92, 255)
    # squat reeds that are plumbing pipes: verticals with elbows + flanges
    for (x, h, bend) in [(6, 14, 1), (11, 19, -1), (16, 11, 1), (21, 17, -1), (26, 13, 1)]:
        y0 = 30 - h
        d.line([x, 30, x, y0 + 3], fill=wood, width=3)
        d.line([x - 1, 30, x - 1, y0 + 3], fill=wood_l, width=1)
        d.arc([x - 3, y0, x + 3 + 3 * bend, y0 + 6], start=270 if bend > 0 else 180,
              end=360 if bend > 0 else 270, fill=wood, width=3)
        ell(d, [x - 2.6, 28.4, x + 2.6, 31], wood_d)  # flange base
        d.point((x + 3 * bend, y0 + 1), fill=wood_d)
    ell(d, [3, 29, 29, 31.6], (70, 52, 32, 255))  # root mat
    return img

def draw_nl_powerbox():
    rng = random.Random(4120)
    img = new_img(); d = ImageDraw.Draw(img)
    green, green_d, rust = (52, 84, 58, 255), (34, 56, 38, 255), (140, 84, 48, 255)
    rect(d, [7, 4, 25, 28], green)
    # organic twisting weld down the center seam
    pts = [(16 + 1.8 * math.sin(y / 2.6), y) for y in range(4, 29, 2)]
    d.line(pts, fill=(150, 130, 84, 255), width=1)
    d.line([7, 16, 25, 16], fill=green_d)  # the two pieces
    for _ in range(8):  # rust bloom
        x = rng.uniform(8, 23); y = rng.uniform(5, 26)
        ell(d, [x, y, x + rng.uniform(1.5, 3.5), y + rng.uniform(1, 2.4)], rust)
    d.line([9, 5, 9, 27], fill=shade(green, 1.35))  # highlight
    for (x, y) in [(8, 5), (24, 5), (8, 27), (24, 27)]:
        d.point((x, y), fill=(30, 30, 30, 255))
    return img

def draw_nl_powerbox_o():
    rng = random.Random(4121)
    img = new_img(); d = ImageDraw.Draw(img)
    green, green_d = (52, 84, 58, 255), (34, 56, 38, 255)
    flesh, flesh_d = (168, 96, 92, 255), (128, 64, 62, 255)
    # swung-open doors on both sides
    d.polygon([(7, 5), (2, 8), (2, 25), (7, 27)], fill=green_d, outline=OL)
    d.polygon([(25, 5), (30, 8), (30, 25), (25, 27)], fill=green_d, outline=OL)
    # cabinet interior: oily fleshy meat with wires
    rect(d, [7, 4, 25, 28], flesh)
    for _ in range(8):
        x = rng.uniform(8, 22); y = rng.uniform(5, 25)
        ell(d, [x, y, x + rng.uniform(2, 5), y + rng.uniform(1.5, 3.5)],
            flesh_d if rng.random() < 0.5 else (196, 128, 116, 255))
    for (x0, y0, x1, y1, col) in [(9, 6, 14, 12, (60, 60, 66, 255)),
                                 (23, 7, 18, 13, (190, 60, 50, 255)),
                                 (9, 25, 13, 20, (70, 110, 170, 255))]:
        d.line([x0, y0, x1, y1], fill=col, width=1)
    # central rectangular depression with connections
    rect(d, [12, 13, 20, 20], (52, 34, 32, 255), (30, 20, 18, 255))
    for (x, y) in [(13, 14), (18, 14), (13, 18), (18, 18)]:
        d.point((x, y), fill=(220, 190, 90, 255))
    d.line([8, 5, 8, 27], fill=(220, 200, 150, 120))  # oily sheen
    return img

# ================================================================ netherum safehouse
def draw_nl_autodoc():
    rng = random.Random(4122)
    img = new_img(); d = ImageDraw.Draw(img)
    body, body_d, accent = (168, 200, 206, 255), (110, 138, 144, 255), (70, 90, 96, 255)
    # barrel base on casters
    rect(d, [11, 18, 21, 28], body)
    d.line([11, 22, 21, 22], fill=body_d)
    d.line([13, 18, 13, 28], fill=shade(body, 1.15))
    for x in (12, 20):
        ell(d, [x - 1.6, 28, x + 1.6, 31], (50, 52, 56, 255), OL)
    ell(d, [13, 15, 19, 19], body_d, OL)  # sensor head
    d.point((16, 17), fill=(255, 90, 80, 255))
    # lanky arms: scalpel arm (left), saw arm (right), grasper (top)
    d.line([12, 19, 6, 12, 4, 6], fill=accent, width=2)
    d.polygon([(4, 6), (2, 2), (6, 4)], fill=(226, 228, 230, 255), outline=OL)
    d.line([20, 19, 26, 13, 28, 8], fill=accent, width=2)
    ell(d, [24, 3, 30, 9], (200, 202, 206, 255), accent)  # circular saw
    d.line([16, 15, 16, 7, 19, 3], fill=accent, width=2)
    d.line([19, 3, 21, 1], fill=accent); d.line([19, 3, 17, 1], fill=accent)
    return img

def draw_nl_autodoc_couch():
    rng = random.Random(4123)
    img = new_img(); d = ImageDraw.Draw(img)
    metal, metal_d = (156, 158, 162, 255), (104, 106, 110, 255)
    # table top + frame on wheels
    rect(d, [3, 12, 29, 16], metal)
    d.line([3, 13, 29, 13], fill=shade(metal, 1.2))
    rect(d, [5, 16, 27, 19], metal_d)
    for x in (7, 25):
        d.line([x, 19, x, 26], fill=metal_d, width=2)
        ell(d, [x - 2, 26, x + 2, 30], (52, 52, 56, 255), OL)
    # dangling black screens with burned-in lines
    for (x, y) in [(6, 19), (21, 20)]:
        d.line([x + 2, 19, x + 2, y], fill=(60, 60, 64, 255))
        rect(d, [x, y, x + 5, y + 4], (14, 14, 16, 255), (80, 82, 86, 255))
        d.line([x + 1, y + 2, x + 4, y + 2], fill=(70, 90, 70, 255))
    # auxiliary devices at the head end
    rect(d, [24, 8, 28, 12], (120, 140, 130, 255), OL)
    d.point((26, 10), fill=(120, 255, 140, 255))
    return img

def draw_nl_boiler():
    rng = random.Random(4124)
    img = new_img(); d = ImageDraw.Draw(img)
    shell, shell_d = (206, 192, 150, 255), (168, 152, 108, 255)  # clouded, yellowed
    # massive cylinder, bolted top and bottom
    rect(d, [6, 3, 26, 29], shell)
    ell(d, [6, 1, 26, 6], shade(shell, 1.1), OL)
    rect(d, [6, 26, 26, 29], shell_d)
    for x in range(8, 26, 4):
        d.point((x, 4), fill=(70, 64, 50, 255))
        d.point((x, 27), fill=(52, 48, 38, 255))
    # the bulbous thing inside, seen murkily through the plastic
    ell(d, [10, 10, 23, 24], (150, 128, 96, 255))
    ell(d, [13, 7, 20, 13], (158, 136, 102, 255))
    d.line([16, 13, 16, 22], fill=(138, 116, 86, 255))
    # clouding streaks + filth where it meets the floor
    for _ in range(7):
        x = rng.uniform(7, 24)
        d.line([x, rng.uniform(5, 10), x + rng.uniform(-1, 1), rng.uniform(16, 24)],
               fill=(226, 216, 178, 255))
    for _ in range(8):
        x = rng.uniform(6, 25); y = rng.uniform(24, 28)
        d.point((x, y), fill=(104, 92, 60, 255))
    d.line([8, 6, 8, 25], fill=(236, 228, 190, 255))  # highlight
    return img

def draw_nl_safehouse_boiler():
    rng = random.Random(4125)
    img = new_img(); d = ImageDraw.Draw(img)
    red, red_d, iron = (156, 52, 44, 255), (108, 34, 30, 255), (56, 54, 58, 255)
    # pot-bellied body
    ell(d, [5, 9, 27, 29], red, OL)
    rect(d, [5, 14, 27, 24], red)
    ell(d, [5, 9, 27, 16], shade(red, 1.1), OL)
    # three unmarked gauges like alien eyes
    for cx in (10, 16, 22):
        ell(d, [cx - 2.4, 4, cx + 2.4, 9], (232, 230, 220, 255), OL)
        d.line([cx, 6.5, cx + 1.2, 5.2], fill=(180, 40, 40, 255))
    # hinged cage over the combustion chamber
    rect(d, [11, 18, 21, 27], (30, 22, 20, 255), iron)
    for x in (13, 16, 19):
        d.line([x, 18, x, 27], fill=iron)
    d.line([11, 22, 21, 22], fill=iron)
    d.point((16, 24), fill=(255, 140, 50, 255))  # firelight within
    # side crank + coiled copper hose (no exhaust vent)
    d.line([27, 14, 30, 14], fill=iron, width=2)
    d.line([30, 14, 30, 18], fill=iron, width=2)
    for i in range(3):
        d.arc([1, 12 + i * 4, 7, 18 + i * 4], start=90, end=270,
              fill=(184, 116, 62, 255), width=1)
    return img

# ================================================================ mutant trees
def _mutant_tree(seed, stage):
    rng = random.Random(seed)
    img = new_img(); d = ImageDraw.Draw(img)
    bark, bark_l, leaf = (92, 70, 44, 255), (122, 96, 58, 255), (96, 168, 74, 255)
    if stage == 0:  # sprout: a tiny little sprout
        d.line([16, 30, 16, 24], fill=(84, 130, 54, 255), width=1)
        ell(d, [10, 20, 16, 25], leaf, OL)
        ell(d, [16, 19, 22, 24], shade(leaf, 1.12), OL)
        ell(d, [12, 29, 20, 31.4], (96, 70, 44, 255))  # soil nub
        return img
    if stage == 1:  # sapling: twisted and gnarled little tree
        pts = [(16 + 2.4 * math.sin(y / 4.0), y) for y in range(30, 13, -2)]
        d.line(pts, fill=bark, width=2)
        for (ex, ey) in [(9, 9), (23, 8), (16, 6)]:
            d.line([pts[-1][0], 14, ex, ey], fill=bark, width=1)
            for _ in range(4):
                x = ex + rng.uniform(-2.4, 2.4); y = ey + rng.uniform(-2, 2)
                ell(d, [x - 1.6, y - 1.1, x + 1.6, y + 1.1], leaf)
        return img
    # adult: twisted branches at odd angles, arms raised like a figure
    d.line([16, 30, 15, 18], fill=bark, width=3)
    d.line([15, 30, 15, 18], fill=bark_l, width=1)
    branches = [(15, 19, 5, 8, 3, 3), (15, 18, 12, 6, 11, 2), (16, 18, 21, 5, 22, 2),
                (16, 19, 27, 9, 29, 4), (15, 21, 8, 14, 6, 10)]
    for (x0, y0, x1, y1, ex, ey) in branches:  # odd angles: kinked mid-joints
        mx, my = (x0 + x1) / 2 + rng.uniform(-1.5, 1.5), (y0 + y1) / 2
        d.line([x0, y0, mx, my, x1, y1], fill=bark, width=2)
        d.line([x1, y1, ex, ey], fill=bark, width=1)
        for _ in range(4):
            x = ex + rng.uniform(-2.6, 2.6); y = ey + rng.uniform(-2.2, 2.2)
            ell(d, [x - 1.7, y - 1.1, x + 1.7, y + 1.1],
                leaf if rng.random() < 0.7 else (140, 190, 84, 255))
    for rx in (11, 21):  # root feet
        d.line([15, 29, rx, 31], fill=bark, width=2)
    return img

def draw_mutant_sprout():
    return _mutant_tree(4126, 0)

def draw_mutant_sapling():
    return _mutant_tree(4127, 1)

def draw_mutant_adult():
    return _mutant_tree(4128, 2)

# ================================================================ tail: appliances / emitters / fakes
def draw_solar_unit_v2():
    rng = random.Random(4129)
    img = new_img(); d = ImageDraw.Draw(img)
    steel = (120, 122, 126, 255)
    # mount post + tilted monocrystalline panel
    d.line([16, 31, 16, 20], fill=steel, width=3)
    d.line([10, 31, 22, 31], fill=shade(steel, 0.75), width=2)
    d.polygon([(4, 8), (28, 4), (28, 17), (4, 21)], fill=(30, 52, 110, 255), outline=OL)
    # monocrystalline cell grid: dark blue cells with rounded corners feel
    for r in range(3):
        for c in range(5):
            x = 6 + c * 4.4; y = 9.6 + r * 3.4 - c * 0.28
            d.rectangle([x, y - r * 0.9, x + 3.4, y + 2.2 - r * 0.9],
                        fill=(44, 76, 152, 255))
    d.polygon([(6, 9), (14, 7.6), (10, 12), (6, 12.6)],
              fill=(120, 160, 220, 110))  # sheen
    return img

def draw_fog_breather():
    rng = random.Random(4130)
    img = new_img()
    ov = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(ov)
    # a misty cloud of water vapor (pink): soft overlapping puffs
    for _ in range(16):
        x = rng.uniform(5, 24); y = rng.uniform(9, 24); r = rng.uniform(3.5, 7.5)
        d.ellipse([x - r, y - r * 0.6, x + r, y + r * 0.6],
                  fill=(246, 198, 218, 68))
    for _ in range(8):
        x = rng.uniform(7, 22); y = rng.uniform(11, 22); r = rng.uniform(2, 4)
        d.ellipse([x - r, y - r * 0.6, x + r, y + r * 0.6],
                  fill=(252, 226, 238, 88))
    img.alpha_composite(ov)
    return img

def _bug_tile(seed):
    img = new_img(); d = ImageDraw.Draw(img)
    d.rectangle([0, 0, S - 1, S - 1], fill=(4, 4, 6, 255))
    # "seeing this is a bug": a magenta warning mark
    d.text((13, 8), "?", fill=(255, 0, 255, 255))
    d.text((12, 18), "BUG", fill=(255, 0, 255, 255))
    return img

def draw_clear():
    return _bug_tile(4131)

def draw_no_item():
    return _bug_tile(4132)

# ================================================================ tail: flora / graves
def draw_dead_flower():
    rng = random.Random(4133)
    img = new_img(); d = ImageDraw.Draw(img)
    # cracked ancient rocky ground
    ell(d, [3, 25, 29, 31.6], (88, 84, 88, 255), OL)
    d.line([7, 28, 13, 27, 18, 29, 25, 27], fill=(52, 50, 54, 255))
    petal = (18, 18, 26, 255)  # black frostbitten petals
    for (x, h) in [(10, 15), (16, 19), (22, 13)]:
        d.line([x, 27, x + rng.uniform(-1, 1), 27 - h], fill=(120, 124, 132, 255), width=1)
        cy = 27 - h
        for a in range(0, 360, 60):  # spindle flower head
            px = x + 3.2 * math.cos(math.radians(a)); py = cy + 2.4 * math.sin(math.radians(a))
            ell(d, [px - 1.5, py - 1.1, px + 1.5, py + 1.1], petal)
        d.point((x, cy), fill=(200, 220, 235, 255))  # frost core
    for _ in range(8):  # glisten
        d.point((rng.uniform(4, 28), rng.uniform(24, 30)), fill=(215, 228, 240, 255))
    return img

def draw_flower_marloss():
    rng = random.Random(4134)
    img = new_img(); d = ImageDraw.Draw(img)
    stem = (66, 120, 52, 255)
    d.line([16, 30, 15, 15], fill=stem, width=2)
    ell(d, [8, 22, 14, 26], (84, 140, 58, 255), OL)  # leaves
    ell(d, [17, 20, 23, 24], (84, 140, 58, 255), OL)
    # petals taken by the mushrooms: pale, cupping the cyan bulb
    for a in range(0, 360, 45):
        px = 15 + 5 * math.cos(math.radians(a)); py = 11 + 4 * math.sin(math.radians(a))
        ell(d, [px - 2.4, py - 1.7, px + 2.4, py + 1.7], (214, 208, 196, 255), OL)
    # the brilliant cyan bulb
    ell(d, [11, 6, 20, 14], (40, 220, 235, 255), OL)
    ell(d, [13, 7.5, 17, 11], (170, 250, 255, 255))
    for _ in range(6):  # alluring aroma motes
        d.point((rng.uniform(6, 25), rng.uniform(3, 16)), fill=(140, 240, 248, 255))
    return img

def draw_grave_monument():
    rng = random.Random(4135)
    img = new_img(); d = ImageDraw.Draw(img)
    stone, stone_d = (172, 172, 178, 255), (118, 118, 124, 255)
    # stepped base + obelisk shaft + pyramid cap
    rect(d, [5, 27, 27, 30], stone_d)
    rect(d, [8, 24, 24, 27], stone)
    d.polygon([(11, 24), (21, 24), (19, 6), (13, 6)], fill=stone, outline=OL)
    d.polygon([(13, 6), (19, 6), (16, 2)], fill=shade(stone, 1.12), outline=OL)
    d.line([13.5, 23, (13.5 + 19) / 2 + 1.5, 7], fill=shade(stone, 1.14))  # highlight edge
    # engraved plaque on the base of the shaft
    rect(d, [13, 17, 19, 22], (140, 128, 84, 255), OL)
    for y in (19, 20.6):
        d.line([14.2, y, 17.8, y], fill=(96, 86, 54, 255))
    return img

# ================================================================ tail: industrial / plumbing
def draw_metal_detector():
    rng = random.Random(4136)
    img = new_img(); d = ImageDraw.Draw(img)
    col, col_d = (168, 196, 214, 255), (110, 138, 162, 255)
    # walk-through arch: two columns + top beam (dead unit, no glow)
    rect(d, [4, 6, 8, 29], col)
    rect(d, [24, 6, 28, 29], col)
    rect(d, [4, 3, 28, 7], col)
    for x in (4, 24):
        d.line([x + 1, 7, x + 1, 28], fill=shade(col, 1.15))
        for y in (11, 17, 23):  # sensor panels
            rect(d, [x + 1, y, x + 3, y + 3], col_d)
    rect(d, [14, 4, 18, 6], (70, 72, 76, 255))  # dead indicator
    rect(d, [3, 29, 9, 31], col_d); rect(d, [23, 29, 29, 31], col_d)
    return img

def draw_weather_station():
    rng = random.Random(4137)
    img = new_img(); d = ImageDraw.Draw(img)
    case, case_l, steel = (34, 34, 40, 255), (66, 66, 74, 255), (140, 142, 148, 255)
    # telescoping stand legs
    for (x0, x1) in [(14, 7), (15, 15), (16, 24)]:
        d.line([x0, 22, x1, 30], fill=steel, width=1)
    # sleek black case, open, devices deployed from the top
    rect(d, [9, 17, 23, 24], case)
    d.line([10, 18, 22, 18], fill=case_l)
    rect(d, [11, 20, 15, 22], (90, 120, 90, 255))  # analysis screen
    d.line([18, 21, 21, 21], fill=(200, 200, 190, 255))  # printer slot
    # telescoping mast + antenna + wind cups
    d.line([16, 17, 16, 4], fill=steel, width=1)
    d.line([11, 5, 21, 5], fill=steel)
    for (x, y) in [(9, 8), (23, 8), (16, 2)]:
        ell(d, [x - 1.4, y - 1.4, x + 1.4, y + 1.4], (200, 202, 206, 255), OL)
    d.line([16, 9, 20, 12], fill=steel)  # small dish arm
    d.arc([18, 10, 24, 15], start=300, end=120, fill=(200, 202, 206, 255))
    return img

def draw_sink_gravity():
    rng = random.Random(4138)
    img = new_img(); d = ImageDraw.Draw(img)
    porc, porc_d, steel = (238, 238, 240, 255), (176, 178, 184, 255), (130, 132, 136, 255)
    # suspended tank above, on straps, with feed pipe
    rect(d, [10, 2, 22, 9], (206, 214, 220, 255), OL)
    d.line([10, 5, 22, 5], fill=porc_d)
    d.line([12, 2, 12, 0], fill=steel); d.line([20, 2, 20, 0], fill=steel)
    d.line([16, 9, 16, 14], fill=steel, width=2)
    # hand pump on the side
    d.line([22, 12, 26, 9], fill=steel, width=2)
    ell(d, [25, 7, 28, 10], (90, 90, 96, 255), OL)
    # basin on a stand
    d.polygon([(7, 15), (25, 15), (22, 21), (10, 21)], fill=porc, outline=OL)
    ell(d, [10, 13.6, 22, 16.4], (188, 205, 214, 255), OL)  # water
    for x in (11, 21):
        d.line([x, 21, x, 29], fill=steel, width=2)
    d.point((16, 14.6), fill=(240, 248, 252, 255))
    return img

# ================================================================ tail: recreation / seats / sleep
def draw_ball_mach():
    rng = random.Random(4139)
    img = new_img(); d = ImageDraw.Draw(img)
    body, body_d = (88, 90, 96, 255), (56, 58, 62, 255)
    # tripod
    for (x0, x1) in [(13, 7), (16, 16), (19, 25)]:
        d.line([x0, 21, x1, 30], fill=body_d, width=2)
    # hopper with balls on top
    d.polygon([(10, 3), (22, 3), (19, 9), (13, 9)], fill=body_d, outline=OL)
    for (x, y) in [(13, 4.6), (17, 4.2), (15, 6.6)]:
        ell(d, [x - 1.6, y - 1.6, x + 1.6, y + 1.6], (238, 238, 240, 255), OL)
    # body with the pair of motorized wheels in their slot
    rect(d, [9, 9, 23, 22], body)
    rect(d, [11, 12, 21, 18], (28, 28, 32, 255))
    ell(d, [11.6, 12.6, 16.4, 17.4], (150, 60, 50, 255), OL)  # wheels
    ell(d, [15.6, 12.6, 20.4, 17.4], (150, 60, 50, 255), OL)
    d.point((14, 15), fill=(30, 30, 34, 255)); d.point((18, 15), fill=(30, 30, 34, 255))
    d.line([10, 10, 10, 21], fill=shade(body, 1.3))
    return img

def draw_roulette_table():
    rng = random.Random(4140)
    img = new_img(); d = ImageDraw.Draw(img)
    wood, felt = (96, 60, 34, 255), (34, 110, 52, 255)
    # huge table filling the tile: wooden rim, green felt top
    d.rounded_rectangle([1, 5, 31, 27], radius=5, fill=wood, outline=OL)
    d.rounded_rectangle([3, 7, 29, 25], radius=4, fill=felt)
    # the wheel: concave circle at the left end
    ell(d, [4.5, 9, 14.5, 21], (120, 30, 30, 255), OL)
    ell(d, [6.5, 11, 12.5, 19], (40, 40, 44, 255), OL)
    ell(d, [8.4, 13.4, 10.6, 16.6], (196, 168, 92, 255), OL)
    d.point((7.4, 12.2), fill=(245, 245, 245, 255))  # the ball
    # betting grid painted on the felt
    for r in range(3):
        for c in range(4):
            x = 17 + c * 3; y = 10 + r * 4
            col = (200, 40, 40, 255) if (r + c) % 2 == 0 else (24, 24, 28, 255)
            rect(d, [x, y, x + 2.2, y + 3], col, (230, 230, 220, 255))
    d.line([17, 23, 28, 23], fill=(230, 230, 220, 255))
    return img

def draw_barber_chair():
    rng = random.Random(4141)
    img = new_img(); d = ImageDraw.Draw(img)
    cloth, cloth_d, chrome = (84, 86, 92, 255), (58, 60, 66, 255), (196, 198, 204, 255)
    # headrest + tall back
    rect(d, [13, 2, 19, 5], cloth_d)
    d.line([16, 5, 16, 7], fill=chrome, width=1)
    d.polygon([(11, 7), (21, 7), (22, 16), (10, 16)], fill=cloth, outline=OL)
    # seat + armrests
    rect(d, [9, 16, 23, 19], cloth)
    d.line([9, 18, 6, 14], fill=chrome, width=1)
    d.line([23, 18, 26, 14], fill=chrome, width=1)
    # hydraulic pedestal + footrest
    d.line([16, 19, 16, 25], fill=chrome, width=2)
    ell(d, [11, 25, 21, 28], cloth_d, OL)
    rect(d, [11, 21, 17, 23], None, chrome)
    return img

def draw_beanbag():
    rng = random.Random(4142)
    img = new_img(); d = ImageDraw.Draw(img)
    fab, fab_d = (128, 96, 116, 255), (96, 70, 88, 255)
    # a large sagging cushion: blob with a sat-in dimple
    ell(d, [4, 10, 28, 29], fab, OL)
    ell(d, [9, 8, 23, 16], shade(fab, 1.1), OL)
    ell(d, [11, 10, 21, 15], fab_d)  # the dimple
    for _ in range(7):  # bead lumps
        x = rng.uniform(7, 24); y = rng.uniform(17, 26)
        d.point((x, y), fill=fab_d)
    d.arc([6, 12, 26, 28], start=200, end=320, fill=shade(fab, 1.2))  # seam highlight
    return img

def _bed(d, mattress, wood, wood_d, pillow=True):
    rect(d, [3, 12, 29, 24], wood)  # frame
    rect(d, [3, 10, 6, 26], wood_d)  # headboard posts
    rect(d, [26, 10, 29, 26], wood_d)
    rect(d, [5, 13, 27, 21], mattress)
    d.line([5, 16, 27, 16], fill=shade(mattress, 0.82))  # blanket fold
    if pillow:
        rect(d, [6, 13.6, 12, 16.4], (242, 240, 244, 255), shade(mattress, 0.8))
    for x in (5, 27):
        d.line([x, 24, x, 27], fill=wood_d, width=2)

def draw_bed_down():
    rng = random.Random(4143)
    img = new_img(); d = ImageDraw.Draw(img)
    _bed(d, (196, 60, 140, 255), (122, 84, 48, 255), (88, 60, 34, 255))
    # downy puffs along the mattress
    for x in range(8, 27, 5):
        d.arc([x, 17, x + 4, 21], start=180, end=360, fill=(222, 110, 172, 255))
    return img

def draw_bunkbed_down():
    rng = random.Random(4144)
    img = new_img(); d = ImageDraw.Draw(img)
    wood, wood_d, mat = (122, 84, 48, 255), (88, 60, 34, 255), (196, 60, 140, 255)
    for x in (3, 27):  # tall posts
        rect(d, [x, 2, x + 3, 29], wood_d)
    # top bunk
    rect(d, [3, 8, 29, 13], wood)
    rect(d, [5, 9, 27, 12], mat)
    rect(d, [6, 9.4, 11, 11.4], (242, 240, 244, 255))
    d.line([3, 6, 29, 6], fill=wood)  # guard rail
    # bottom bunk
    rect(d, [3, 20, 29, 25], wood)
    rect(d, [5, 21, 27, 24], mat)
    rect(d, [6, 21.4, 11, 23.4], (242, 240, 244, 255))
    # ladder
    for y in (14, 17, 20):
        d.line([23, y, 27, y], fill=wood_d, width=1)
    d.line([23, 13, 23, 21], fill=wood_d); d.line([27, 13, 27, 21], fill=wood_d)
    return img

# ================================================================ tail: tools / military / nether
def draw_firefly_terrarium():
    rng = random.Random(4145)
    img = new_img(); d = ImageDraw.Draw(img)
    # glass tank
    rect(d, [4, 8, 28, 28], (170, 205, 190, 60), (120, 150, 138, 255))
    rect(d, [4, 25, 28, 28], (96, 70, 44, 255))  # soil
    d.line([6, 10, 12, 26], fill=(215, 235, 220, 90))  # glass sheen
    # strange plants
    for (x, h, col) in [(9, 9, (70, 140, 70, 255)), (22, 12, (96, 160, 84, 255)),
                        (16, 7, (120, 150, 70, 255))]:
        d.line([x, 25, x + rng.uniform(-2, 2), 25 - h], fill=col, width=1)
        ell(d, [x - 2, 25 - h - 2, x + 2, 25 - h + 1], col)
    # the giant mutant firefly, glowing abdomen
    ell(d, [13, 13, 19, 18], (52, 44, 36, 255), OL)  # body
    ell(d, [17.4, 14.6, 21, 18.4], (240, 235, 120, 255), OL)  # lit abdomen
    d.polygon([(14, 13), (10, 9), (13, 15)], fill=(210, 220, 215, 130))  # wing
    for _ in range(6):
        d.point((rng.uniform(6, 26), rng.uniform(10, 22)), fill=(250, 245, 160, 255))
    rect(d, [4, 6, 28, 8], (90, 90, 96, 255))  # lid frame
    return img

def draw_screw_press():
    rng = random.Random(4146)
    img = new_img(); d = ImageDraw.Draw(img)
    frame, frame_d, accent = (172, 84, 72, 255), (124, 56, 50, 255), (110, 110, 116, 255)
    # frame: two posts + top beam + base
    rect(d, [6, 6, 9, 28], frame)
    rect(d, [23, 6, 26, 28], frame)
    rect(d, [5, 4, 27, 7], frame_d)
    rect(d, [5, 27, 27, 30], frame_d)
    # the great screw + handle bar
    d.line([16, 7, 16, 17], fill=accent, width=3)
    for y in range(8, 17, 2):
        d.line([14.6, y, 17.4, y + 1], fill=shade(accent, 0.75))
    d.line([9, 9, 23, 9], fill=accent, width=2)
    ell(d, [7.4, 7.6, 10, 10.4], frame_d, OL); ell(d, [22, 7.6, 24.6, 10.4], frame_d, OL)
    # press plate + wetted paste bags
    rect(d, [11, 17, 21, 19], frame_d)
    for i, col in enumerate([(196, 178, 140, 255), (178, 158, 118, 255), (196, 178, 140, 255)]):
        rect(d, [11, 20 + i * 2.2, 21, 22 + i * 2.2], col, frame_d)
    return img

def draw_m224_mortar():
    rng = random.Random(4147)
    img = new_img(); d = ImageDraw.Draw(img)
    od, od_d = (84, 88, 76, 255), (54, 58, 50, 255)
    # baseplate + bipod
    ell(d, [9, 26, 23, 30], od_d, OL)
    d.line([14, 20, 10, 27], fill=od, width=2)
    d.line([19, 20, 23, 27], fill=od, width=2)
    # the 60mm tube, angled up
    d.polygon([(13, 21), (21, 4), (24, 5), (17, 22)], fill=od, outline=OL)
    d.line([15, 20, (15 + 22) / 2, 6], fill=shade(od, 1.3), width=1)
    ell(d, [20.4, 3, 24.4, 6], od_d, OL)  # muzzle
    # firing tables clipped tight beside it
    rect(d, [25, 14, 30, 22], (214, 208, 190, 255), OL)
    for y in (16.4, 18.4, 20.4):
        d.line([26, y, 29, y], fill=(120, 116, 104, 255))
    d.point((27.4, 13.4), fill=(200, 60, 50, 255))  # clip
    return img

def draw_nm_nether_lichen():
    rng = random.Random(4148)
    img = new_img(); d = ImageDraw.Draw(img)
    # bed of lichen covering yellow ichor
    ell(d, [1, 8, 31, 30], (206, 178, 60, 255), OL)  # ichor pool
    for _ in range(26):  # orange lichen patches
        x = rng.uniform(3, 26); y = rng.uniform(10, 26); r = rng.uniform(1.6, 3.6)
        ell(d, [x - r, y - r * 0.6, x + r, y + r * 0.6],
            (226, 128, 40, 255) if rng.random() < 0.6 else (240, 156, 60, 255))
    for _ in range(9):  # tiny lichen flowers
        x = rng.uniform(4, 27); y = rng.uniform(10, 26)
        ell(d, [x - 1.2, y - 1.2, x + 1.2, y + 1.2], (250, 214, 120, 255))
        d.point((x, y), fill=(255, 244, 190, 255))
    d.line([5, 12, 12, 10], fill=(232, 200, 96, 255))  # ichor sheen
    return img

def draw_sunkenhelicopter_seat():
    rng = random.Random(4149)
    img = new_img(); d = ImageDraw.Draw(img)
    fab, fab_d, algae = (88, 116, 74, 255), (60, 82, 50, 255), (70, 140, 84, 255)
    # contoured bucket seat: tall back + cushion, waterlogged and torn
    d.polygon([(10, 4), (22, 4), (24, 20), (8, 20)], fill=fab, outline=OL)
    d.polygon([(11, 6), (21, 6), (22, 12), (10, 12)], fill=fab_d, outline=OL)  # headrest inset
    d.polygon([(7, 20), (25, 20), (26, 26), (6, 26)], fill=fab, outline=OL)  # cushion
    d.line([10, 22.6, 22, 22.6], fill=fab_d)
    for _ in range(7):  # degradation: algae streaks + silt
        x = rng.uniform(8, 23); y = rng.uniform(6, 24)
        d.line([x, y, x + rng.uniform(-1, 1), y + rng.uniform(2, 4)],
               fill=algae if rng.random() < 0.6 else (140, 132, 96, 255))
    d.polygon([(19, 15), (22, 15), (21, 19)], fill=(40, 44, 34, 255))  # tear
    rect(d, [13, 26, 19, 29], (70, 70, 66, 255))  # swivel base remnant
    return img

# ================================================================ jobs
JOBS = [
    ("f_hobo_stove", draw_hobo_stove()),
    ("f_solar_cooker", draw_solar_cooker()),
    ("f_exam_chair", draw_exam_chair()),
    ("f_surgical_table", draw_surgical_table()),
    ("f_butter_churn", draw_butter_churn()),
    ("f_metal_crate_military", draw_metal_crate_military()),
    ("f_metal_crate_military_o", draw_metal_crate_military_o()),
    ("f_torah_ark", draw_torah_ark()),
    ("f_brambles", draw_brambles()),
    ("f_glowingbulb", draw_glowingbulb()),
    ("f_midden", draw_midden()),
    ("f_12000_gasoline_generator", draw_gen_12000()),
    ("f_diesel_generator", draw_gen_diesel()),
    ("f_military_generator", draw_gen_military()),
    ("f_portable_generator", draw_gen_portable()),
    ("f_portal_dungeon_reward_increaser", draw_portal_increaser()),
    ("f_portal_dungeon_reward_increaser_inert", draw_portal_increaser_inert()),
    ("f_nl_climbing_vine", draw_nl_climbing_vine()),
    ("f_nl_pipe_plant", draw_nl_pipe_plant()),
    ("f_nl_powerbox", draw_nl_powerbox()),
    ("f_nl_powerbox_o", draw_nl_powerbox_o()),
    ("f_nl_autodoc", draw_nl_autodoc()),
    ("f_nl_autodoc_couch", draw_nl_autodoc_couch()),
    ("f_nl_boiler", draw_nl_boiler()),
    ("f_nl_safehouse_boiler", draw_nl_safehouse_boiler()),
    ("f_mutant_tree_sprout", draw_mutant_sprout()),
    ("f_mutant_tree_sapling", draw_mutant_sapling()),
    ("f_mutant_tree_adult", draw_mutant_adult()),
    ("f_solar_unit_v2", draw_solar_unit_v2()),
    ("f_fog_breather", draw_fog_breather()),
    ("f_clear", draw_clear()),
    ("f_no_item", draw_no_item()),
    ("f_string_dimension_dead_flower", draw_dead_flower()),
    ("f_flower_marloss", draw_flower_marloss()),
    ("f_grave_monument", draw_grave_monument()),
    ("f_metal_detector", draw_metal_detector()),
    ("f_xedra_mobile_weather_station", draw_weather_station()),
    ("f_sink_gravity", draw_sink_gravity()),
    ("f_ball_mach", draw_ball_mach()),
    ("f_roulette_table", draw_roulette_table()),
    ("f_barber_chair", draw_barber_chair()),
    ("f_beanbag", draw_beanbag()),
    ("f_bed_down", draw_bed_down()),
    ("f_bunkbed_down", draw_bunkbed_down()),
    ("f_firefly_terrarium", draw_firefly_terrarium()),
    ("f_screw_press", draw_screw_press()),
    ("f_m224_mortar", draw_m224_mortar()),
    ("f_nm_nether_lichen", draw_nm_nether_lichen()),
    ("f_sunkenhelicopter_seat_rotating", draw_sunkenhelicopter_seat()),
]

CREDITS = ("Drawn from the furniture's in-game description for the CDDA Voxel\n"
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
CELL = 64
COLS = 7
ROWS = (len(JOBS) + COLS - 1) // COLS
sheet = Image.new("RGBA", (COLS * CELL, ROWS * CELL), (24, 24, 28, 255))
for i, (tid, _img) in enumerate(JOBS):
    p = os.path.join(BASE, "pngs_normal_32x32", "furniture", tid, tid + ".png")
    t = Image.open(p).convert("RGBA").resize((CELL, CELL), Image.NEAREST)
    sheet.alpha_composite(t, ((i % COLS) * CELL, (i // COLS) * CELL))
sheet.save(os.path.join(BASE, "preview_furniture_remainder.png"))
print("wrote preview_furniture_remainder.png", sheet.size)
