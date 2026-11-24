#!/usr/bin/env python3
"""Batch FIELD2 (Phase 2g.4 scope extension): field types, final field batch.

45 ids: 35 zone-manager display fields (data/json/zone_field_type.json) and
10 physical fields (data/json/field_type.json).

Zone fields are organizational area markers, never physical phenomena, so the
whole family shares ONE treatment (FIELD1's draw_field1.py did not exist on
disk when this batch started, so this script establishes it): a transparent
tile carrying a corner-bracketed area square in a per-category accent color,
with a category pictogram in pale paper-white at the center. NPC zones use a
figure glyph plus a sign (slash = forbidden, "?" = investigation, bent arrow
= retreat); perishable zones add a snowflake mark.

Physical fields drawn as themselves:
  fd_last_known            ghost map-pin + dashed ring (last known position)
  fd_migo_atmosphere       sickly violet/green alien air wisps + spores
  fd_nauseating_smell      green stench wisps
  fd_push_items            arrows radiating outward (sym "&", pushes items)
  fd_scarlet_atmosphere    full-tile opaque scarlet haze wash
  fd_scarlet_fog           full-tile opaque dense scarlet fog (non-transparent)
  fd_scarlet_sea_gas       scarlet haze/smoke/smog wisps over a low wash
  fd_scarlet_sea_wave      cresting light-red wave bands (sym "~")
  fd_scarlet_storm_lightning  dark scarlet sky, aurora ribbons + bolt
  fd_shadow                full-tile near-black patch (light_override 3.7)

PIL only, per-sprite seeded random.Random (crc32 of id). Alpha is strictly
{0, 255}: translucency is faked with pale opaque tones. Writes ONLY its own
45 folders + preview_field2.png. No other files touched.
"""
import json
import math
import os
import random
import zlib

from PIL import Image, ImageDraw

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "fields")
CREDITS = ("Drawn from the field's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the "
           "tileset.\n")

PAPER = (232, 230, 218, 255)
PAPER_DIM = (188, 186, 174, 255)
DARK = (38, 38, 44, 255)
SLASH_RED = (216, 48, 44, 255)

# ---------------------------------------------------------------- zone base

def brackets(d, accent):
    """Corner-bracketed area square, inset 3px, arm 7px, 2px thick."""
    dark = tuple(max(0, c - 70) for c in accent[:3]) + (255,)
    for col, off in ((dark, 0), (accent, 0)):
        x0, y0, x1, y1, a, t = 3, 3, 28, 28, 7, 2
        # top-left
        d.rectangle([x0, y0, x0 + a, y0 + t - 1], fill=col)
        d.rectangle([x0, y0, x0 + t - 1, y0 + a], fill=col)
        # top-right
        d.rectangle([x1 - a, y0, x1, y0 + t - 1], fill=col)
        d.rectangle([x1 - t + 1, y0, x1, y0 + a], fill=col)
        # bottom-left
        d.rectangle([x0, y1 - t + 1, x0 + a, y1], fill=col)
        d.rectangle([x0, y1 - a, x0 + t - 1, y1], fill=col)
        # bottom-right
        d.rectangle([x1 - a, y1 - t + 1, x1, y1], fill=col)
        d.rectangle([x1 - t + 1, y1 - a, x1, y1], fill=col)
        if col is dark:
            # second pass draws accent slightly inside
            pass

def slash(d, x0=9, y0=23, x1=23, y1=9):
    d.line([x0, y0, x1, y1], fill=SLASH_RED, width=3)
    d.line([x0, y0 - 1, x1, y1 - 1], fill=SLASH_RED, width=1)

def figure(d, col=PAPER):
    """Simple standing NPC figure, center (16,15)."""
    d.ellipse([13, 7, 19, 13], fill=col)                    # head
    d.rectangle([14, 13, 18, 21], fill=col)                 # torso
    d.line([14, 21, 12, 26], fill=col, width=2)             # legs
    d.line([18, 21, 20, 26], fill=col, width=2)

def snowflake(d, cx, cy, col, r=3):
    for ang in (0, 60, 120):
        dx = int(round(r * math.cos(math.radians(ang))))
        dy = int(round(r * math.sin(math.radians(ang))))
        d.line([cx - dx, cy - dy, cx + dx, cy + dy], fill=col, width=1)

# ------------------------------------------------------------- pictograms
# Each draws inside roughly x 9..23, y 9..23.

def p_box(d, accent, rng):
    d.rectangle([10, 12, 22, 23], outline=PAPER, width=2)
    d.line([10, 16, 22, 16], fill=PAPER, width=2)
    d.line([16, 12, 16, 16], fill=accent, width=2)

def p_boxes_group(d, accent, rng):
    d.rectangle([8, 14, 17, 23], outline=PAPER, width=2)
    d.rectangle([15, 10, 24, 19], outline=accent, width=2)
    d.line([15, 14, 24, 14], fill=accent, width=1)

def p_key(d, accent, rng):
    d.ellipse([9, 9, 17, 17], outline=(250, 205, 60, 255), width=2)
    d.line([15, 15, 23, 23], fill=(250, 205, 60, 255), width=2)
    d.line([19, 19, 22, 19], fill=(250, 205, 60, 255), width=2)
    d.line([21, 21, 21, 18], fill=(250, 205, 60, 255), width=2)

def p_magazine(d, accent, rng):
    d.rectangle([11, 8, 21, 24], fill=PAPER)
    d.polygon([(21, 8), (21, 13), (16, 8)], fill=PAPER_DIM)
    for yy in (16, 19, 22):
        d.line([13, yy, 19, yy], fill=accent, width=1)
    d.rectangle([13, 10, 17, 13], fill=accent)

def p_book(d, accent, rng):
    d.rectangle([10, 9, 22, 24], fill=accent)
    d.rectangle([12, 9, 22, 24], fill=PAPER)
    for yy in (13, 16, 19):
        d.line([14, yy, 20, yy], fill=PAPER_DIM, width=1)
    d.rectangle([10, 9, 12, 24], fill=tuple(max(0, c - 60) for c in accent[:3]) + (255,))

def p_martial_book(d, accent, rng):
    p_book(d, (170, 40, 40, 255), rng)
    # belt stripe across the cover
    d.rectangle([12, 17, 22, 20], fill=DARK)
    d.rectangle([19, 17, 22, 20], fill=(250, 205, 60, 255))

def p_map(d, accent, rng):
    d.polygon([(9, 11), (14, 9), (19, 11), (23, 9), (23, 21), (19, 23),
               (14, 21), (9, 23)], fill=PAPER)
    d.line([14, 9, 14, 21], fill=PAPER_DIM, width=1)
    d.line([19, 11, 19, 23], fill=PAPER_DIM, width=1)
    d.line([11, 19, 16, 14, 21, 17], fill=(216, 48, 44, 255), width=1)
    d.ellipse([19, 15, 22, 18], fill=(216, 48, 44, 255))

def p_pickaxe(d, accent, rng):
    d.arc([8, 6, 26, 20], start=200, end=340, fill=PAPER, width=3)
    d.line([16, 12, 20, 24], fill=(160, 110, 60, 255), width=3)
    d.polygon([(8, 22), (24, 22), (21, 25), (11, 25)], fill=accent)

def p_gear(d, accent, rng, cx=16, cy=16, r=6, col=None):
    col = col or PAPER
    for k in range(8):
        a = math.radians(k * 45)
        x = cx + int(round((r + 1) * math.cos(a)))
        y = cy + int(round((r + 1) * math.sin(a)))
        d.rectangle([x - 1, y - 1, x + 1, y + 1], fill=col)
    d.ellipse([cx - r, cy - r, cx + r, cy + r], outline=col, width=2)
    d.ellipse([cx - 2, cy - 2, cx + 2, cy + 2], fill=col)

def p_mods(d, accent, rng):
    p_gear(d, accent, rng, cx=14, cy=16, r=5)
    d.line([21, 10, 21, 16], fill=accent, width=2)   # plus = attachment
    d.line([18, 13, 24, 13], fill=accent, width=2)

def p_mop(d, accent, rng):
    d.line([19, 8, 13, 20], fill=(160, 110, 60, 255), width=2)
    for dx in (-3, -1, 1, 3):
        d.line([13 + dx // 2, 20, 12 + dx, 25], fill=accent, width=2)

def p_flask(d, accent, rng):
    d.polygon([(14, 8), (18, 8), (18, 13), (22, 22), (22, 24), (10, 24),
               (10, 22), (14, 13)], fill=PAPER)
    d.polygon([(12, 19), (20, 19), (21, 23), (11, 23)], fill=accent)
    for (bx, by) in ((14, 21), (17, 20), (16, 22)):
        d.point((bx, by), fill=PAPER)
    d.rectangle([13, 7, 19, 8], fill=PAPER_DIM)

def p_eye_slash(d, accent, rng):
    d.arc([8, 10, 24, 22], start=0, end=360, fill=PAPER, width=2)
    d.polygon([(8, 16), (16, 10), (24, 16), (16, 22)], outline=PAPER, width=2)
    d.ellipse([14, 14, 18, 18], fill=accent)
    slash(d)

def p_box_slash(d, accent, rng):
    p_box(d, accent, rng)
    slash(d)

def p_figure_slash(d, accent, rng):
    figure(d)
    slash(d, 8, 24, 24, 8)

def p_figure_ear_slash(d, accent, rng):
    figure(d)
    for r in (4, 7):
        d.arc([19, 6, 19 + r * 2, 6 + r * 2], start=300, end=60, fill=accent, width=1)
    slash(d, 18, 22, 26, 10)

def p_figure_question(d, accent, rng):
    figure(d)
    # pixel "?" to the upper right
    q = (250, 205, 60, 255)
    d.arc([19, 5, 25, 11], start=180, end=360, fill=q, width=2)
    d.line([25, 8, 22, 11], fill=q, width=2)
    d.line([22, 11, 22, 13], fill=q, width=2)
    d.point((22, 16), fill=q)
    d.rectangle([21, 15, 23, 17], fill=q)

def p_figure_retreat(d, accent, rng):
    figure(d)
    # bent retreat arrow, lower left, pointing back/down
    d.line([13, 24, 7, 24], fill=accent, width=2)
    d.line([7, 24, 7, 19], fill=accent, width=2)
    d.polygon([(4, 21), (10, 21), (7, 17)], fill=accent)

def p_other(d, accent, rng):
    p_box(d, accent, rng)
    for dx in (-4, 0, 4):
        d.ellipse([15 + dx, 18, 17 + dx, 20], fill=accent)

def p_drink_perishable(d, accent, rng):
    d.rectangle([13, 8, 19, 24], fill=PAPER)          # bottle
    d.rectangle([14, 6, 18, 8], fill=PAPER_DIM)
    d.rectangle([13, 15, 19, 24], fill=(90, 170, 230, 255))
    snowflake(d, 22, 10, (170, 225, 255, 255), 3)

def p_food_perishable(d, accent, rng):
    d.ellipse([10, 12, 20, 23], fill=(120, 200, 90, 255))   # apple
    d.line([15, 12, 17, 8], fill=(160, 110, 60, 255), width=2)
    d.polygon([(17, 8), (21, 8), (18, 11)], fill=(90, 170, 80, 255))
    snowflake(d, 22, 20, (170, 225, 255, 255), 3)

def p_sprout(d, accent, rng):
    d.line([16, 24, 16, 15], fill=(110, 190, 80, 255), width=2)
    d.polygon([(16, 15), (9, 15), (10, 9), (16, 11)], fill=(110, 190, 80, 255))
    d.polygon([(16, 14), (23, 14), (22, 8), (16, 10)], fill=(140, 215, 100, 255))
    d.line([11, 25, 21, 25], fill=(160, 110, 60, 255), width=2)

def p_spare_parts(d, accent, rng):
    p_gear(d, accent, rng, cx=13, cy=14, r=5)
    d.rectangle([18, 18, 23, 21], fill=PAPER)          # bolt block
    d.line([19, 19, 22, 19], fill=DARK, width=1)
    d.line([20, 16, 20, 18], fill=PAPER, width=2)      # bolt shaft

def p_flag(d, accent, rng):
    d.line([11, 6, 11, 26], fill=PAPER, width=2)
    d.polygon([(11, 6), (22, 9), (11, 13)], fill=(90, 220, 110, 255))
    d.line([8, 26, 15, 26], fill=PAPER_DIM, width=2)

def p_corpse(d, accent, rng):
    # prone body, head left
    d.ellipse([7, 14, 13, 20], fill=PAPER)
    d.rectangle([13, 14, 24, 19], fill=PAPER_DIM)
    d.line([20, 19, 25, 22], fill=PAPER, width=2)      # legs sprawled
    d.line([21, 19, 24, 24], fill=PAPER, width=2)
    d.line([14, 19, 11, 23], fill=PAPER, width=2)      # arm
    d.rectangle([15, 15, 19, 18], outline=accent)      # shirt being stripped

def p_tool_magazine(d, accent, rng):
    p_magazine(d, accent, rng)
    d.line([19, 24, 23, 20], fill=DARK, width=2)       # wrench nub
    d.arc([21, 17, 26, 22], start=300, end=120, fill=DARK, width=2)

def p_tools(d, accent, rng):
    d.line([11, 22, 20, 13], fill=PAPER, width=3)      # wrench shaft
    d.arc([17, 8, 24, 15], start=300, end=130, fill=PAPER, width=3)
    d.line([12, 12, 20, 20], fill=(240, 150, 50, 255), width=3)  # driver
    d.line([19, 19, 22, 22], fill=PAPER_DIM, width=2)

def p_trapjaw(d, accent, rng):
    d.arc([9, 12, 23, 26], start=180, end=360, fill=PAPER, width=2)
    for x in range(11, 22, 3):                          # teeth
        d.polygon([(x, 14), (x + 2, 14), (x + 1, 10)], fill=PAPER)
    d.ellipse([14, 17, 18, 21], fill=accent)

def p_unload(d, accent, rng):
    d.rectangle([10, 15, 22, 24], outline=PAPER, width=2)
    d.line([16, 14, 16, 8], fill=accent, width=2)      # arrow up out of box
    d.polygon([(13, 10), (19, 10), (16, 6)], fill=accent)

def p_pile(d, accent, rng):
    d.polygon([(8, 24), (12, 15), (17, 18), (20, 12), (24, 24)], fill=PAPER_DIM)
    d.ellipse([12, 15, 16, 19], fill=(200, 90, 80, 255))
    d.rectangle([17, 18, 21, 22], fill=(90, 140, 200, 255))
    d.polygon([(19, 11), (22, 16), (17, 16)], fill=(240, 200, 80, 255))

def p_car(d, accent, rng):
    d.polygon([(8, 19), (10, 19), (12, 14), (20, 14), (22, 19), (24, 19),
               (24, 22), (8, 22)], fill=PAPER)
    d.rectangle([13, 15, 16, 18], fill=DARK)
    d.rectangle([17, 15, 20, 18], fill=DARK)
    d.ellipse([10, 21, 14, 25], fill=DARK)
    d.ellipse([18, 21, 22, 25], fill=DARK)

def p_vehicle_deconstruct(d, accent, rng):
    p_car(d, accent, rng)
    d.line([10, 9, 22, 21], fill=accent, width=2)      # dismantle cross
    d.line([22, 9, 10, 21], fill=accent, width=1)

def p_vehicle_parts(d, accent, rng):
    d.ellipse([9, 9, 23, 23], outline=DARK, width=3)   # tire
    d.ellipse([13, 13, 19, 19], outline=PAPER, width=2)
    d.ellipse([15, 15, 17, 17], fill=accent)
    for a in (0, 90, 180, 270):
        dx = int(3 * math.cos(math.radians(a)))
        dy = int(3 * math.sin(math.radians(a)))
        d.point((16 + dx, 16 + dy), fill=PAPER)

def p_vehicle_patrol(d, accent, rng):
    p_car(d, accent, rng)
    d.arc([6, 4, 26, 24], start=40, end=140, fill=accent, width=2)
    d.polygon([(8, 7), (13, 6), (10, 11)], fill=accent)  # route arrowhead

def p_vehicle_repair(d, accent, rng):
    p_car(d, accent, rng)
    d.line([20, 6, 24, 10], fill=(240, 150, 50, 255), width=2)
    d.arc([22, 4, 27, 9], start=300, end=120, fill=(240, 150, 50, 255), width=2)

def p_weapon(d, accent, rng):
    d.line([9, 21, 19, 11], fill=PAPER, width=3)       # rifle body diagonal
    d.line([19, 11, 23, 9], fill=PAPER, width=2)       # barrel
    d.line([13, 17, 16, 21], fill=(160, 110, 60, 255), width=2)  # grip
    d.rectangle([10, 20, 14, 23], fill=(160, 110, 60, 255))      # stock

def p_wood(d, accent, rng):
    for (x, y) in ((11, 13), (18, 13), (14, 19)):
        d.ellipse([x - 4, y - 4, x + 4, y + 4], fill=(170, 120, 70, 255))
        d.ellipse([x - 4, y - 4, x + 4, y + 4], outline=(120, 80, 40, 255))
        d.ellipse([x - 1, y - 1, x + 1, y + 1], outline=(120, 80, 40, 255))
    d.rectangle([8, 22, 24, 24], fill=(140, 95, 50, 255))

ZONES = {
    # id: (accent, pictogram)
    "fd_ignore_loot_zone": ((150, 150, 160, 255), p_eye_slash),
    "fd_item_group_loot_zone": ((200, 160, 90, 255), p_boxes_group),
    "fd_keys_loot_zone": ((250, 205, 60, 255), p_key),
    "fd_magazines_loot_zone": ((90, 200, 210, 255), p_magazine),
    "fd_manuals_loot_zone": ((90, 130, 230, 255), p_book),
    "fd_maps_loot_zone": ((110, 200, 110, 255), p_map),
    "fd_martial_arts_manuals_loot_zone": ((220, 70, 60, 255), p_martial_book),
    "fd_mine_terrain_zone": ((230, 140, 60, 255), p_pickaxe),
    "fd_mods_loot_zone": ((180, 110, 230, 255), p_mods),
    "fd_mop_tile_zone": ((130, 190, 240, 255), p_mop),
    "fd_mutagens_loot_zone": ((170, 90, 220, 255), p_flask),
    "fd_no_auto_pickup_zone": ((216, 80, 70, 255), p_box_slash),
    "fd_no_npc_pickup_zone": ((216, 80, 70, 255), p_figure_slash),
    "fd_npc_ignore_sounds_zone": ((200, 120, 120, 255), p_figure_ear_slash),
    "fd_npc_investigation_area_zone": ((250, 205, 60, 255), p_figure_question),
    "fd_npc_no_go_zone": ((240, 60, 50, 255), p_figure_slash),
    "fd_npc_retreat_zone": ((240, 160, 60, 255), p_figure_retreat),
    "fd_other_loot_zone": ((160, 170, 190, 255), p_other),
    "fd_perishable_drink_loot_zone": ((120, 200, 240, 255), p_drink_perishable),
    "fd_perishable_food_loot_zone": ((120, 220, 190, 255), p_food_perishable),
    "fd_seeds_loot_zone": ((120, 210, 90, 255), p_sprout),
    "fd_spare_parts_loot_zone": ((150, 170, 200, 255), p_spare_parts),
    "fd_start_point_zone": ((110, 230, 130, 255), p_flag),
    "fd_strip_corpses_loot_zone": ((190, 180, 170, 255), p_corpse),
    "fd_tool_magazines_loot_zone": ((80, 190, 170, 255), p_tool_magazine),
    "fd_tools_loot_zone": ((240, 150, 50, 255), p_tools),
    "fd_traps_loot_zone": ((200, 100, 70, 255), p_trapjaw),
    "fd_unload_everything_loot_zone": ((210, 170, 100, 255), p_unload),
    "fd_unsorted_loot_zone": ((170, 160, 150, 255), p_pile),
    "fd_vehicle_deconstruct_zone": ((200, 120, 90, 255), p_vehicle_deconstruct),
    "fd_vehicle_parts_loot_zone": ((170, 180, 200, 255), p_vehicle_parts),
    "fd_vehicle_patrol_zone": ((100, 160, 240, 255), p_vehicle_patrol),
    "fd_vehicle_repair_zone": ((240, 130, 70, 255), p_vehicle_repair),
    "fd_weapons_loot_zone": ((220, 90, 80, 255), p_weapon),
    "fd_wood_loot_zone": ((180, 130, 80, 255), p_wood),
}

# --------------------------------------------------------- physical fields

def blob(d, cx, cy, r, col, rng, wobble=0.35):
    pts = []
    for k in range(10):
        a = math.radians(k * 36)
        rr = r * (1 + rng.uniform(-wobble, wobble))
        pts.append((cx + rr * math.cos(a), cy + rr * math.sin(a)))
    d.polygon(pts, fill=col)

def f_shadow(rng):
    img = Image.new("RGBA", (32, 32), (6, 6, 10, 255))
    d = ImageDraw.Draw(img)
    for _ in range(26):
        blob(d, rng.uniform(0, 32), rng.uniform(0, 32), rng.uniform(3, 7),
             (14, 14, 22, 255), rng)
    for _ in range(10):
        blob(d, rng.uniform(0, 32), rng.uniform(0, 32), rng.uniform(2, 4),
             (2, 2, 5, 255), rng)
    return img

def wash(rng, base, tones, n=30):
    img = Image.new("RGBA", (32, 32), base)
    d = ImageDraw.Draw(img)
    for _ in range(n):
        blob(d, rng.uniform(-2, 34), rng.uniform(-2, 34), rng.uniform(4, 9),
             rng.choice(tones), rng)
    return img, d

def f_scarlet_atmosphere(rng):
    img, d = wash(rng, (148, 34, 34, 255),
                  [(168, 46, 42, 255), (128, 28, 30, 255), (178, 60, 52, 255)])
    for _ in range(14):  # drifting motes
        x, y = rng.uniform(1, 30), rng.uniform(1, 30)
        d.ellipse([x - 1, y - 1, x + 1, y + 1], fill=(235, 120, 105, 255))
    return img

def f_scarlet_fog(rng):
    img, d = wash(rng, (172, 44, 40, 255),
                  [(196, 66, 58, 255), (150, 36, 36, 255),
                   (214, 92, 80, 255), (132, 30, 30, 255)], n=44)
    for _ in range(8):  # denser bright cores (light_emitted red glow)
        blob(d, rng.uniform(4, 28), rng.uniform(4, 28), rng.uniform(2, 4),
             (228, 110, 96, 255), rng)
    return img

def wisps(d, rng, col_a, col_b, n=5, ybase=26):
    for i in range(n):
        x = 4 + i * 6 + rng.uniform(-1, 1)
        amp = rng.uniform(1.5, 3.0)
        pts = []
        for t in range(17):
            yy = ybase - t
            xx = x + amp * math.sin(t / 3.2 + i)
            pts.append((xx, yy))
        d.line(pts, fill=col_a if i % 2 == 0 else col_b, width=2)

def f_scarlet_sea_gas(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # low haze wash (haze -> smoke -> smog graded density)
    for _ in range(16):
        blob(d, rng.uniform(0, 32), rng.uniform(22, 33), rng.uniform(4, 8),
             (196, 80, 70, 255), rng)
    wisps(d, rng, (214, 92, 80, 255), (168, 52, 48, 255), n=5, ybase=27)
    return img

def f_migo_atmosphere(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    wisps(d, rng, (196, 140, 210, 255), (150, 190, 120, 255), n=5, ybase=28)
    for _ in range(18):  # alien spores
        x, y = rng.uniform(1, 30), rng.uniform(1, 30)
        c = rng.choice([(220, 170, 230, 255), (170, 220, 140, 255)])
        d.ellipse([x - 1, y - 1, x + 1, y + 1], fill=c)
    return img

def f_nauseating_smell(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    wisps(d, rng, (130, 200, 80, 255), (95, 160, 60, 255), n=5, ybase=28)
    for _ in range(8):
        x, y = rng.uniform(2, 29), rng.uniform(18, 29)
        d.ellipse([x - 1, y - 1, x + 1, y + 1], fill=(160, 220, 110, 255))
    return img

def f_scarlet_sea_wave(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    bands = [(7, (150, 40, 40, 255)), (15, (186, 60, 54, 255)),
             (23, (214, 92, 80, 255))]
    for (yy, col) in bands:
        pts = []
        for x in range(33):
            pts.append((x, yy + 2.2 * math.sin(x / 4.5 + yy)))
        d.line(pts, fill=col, width=3)
        # foam caps on crests
        for x in range(2, 32, 7):
            cy = yy + 2.2 * math.sin(x / 4.5 + yy)
            d.ellipse([x - 1, cy - 3, x + 2, cy], fill=(245, 170, 158, 255))
    return img

def f_scarlet_storm_lightning(rng):
    img = Image.new("RGBA", (32, 32), (52, 10, 14, 255))
    d = ImageDraw.Draw(img)
    for _ in range(12):
        blob(d, rng.uniform(0, 32), rng.uniform(0, 32), rng.uniform(3, 7),
             (74, 16, 20, 255), rng)
    # aurora ribbons
    for (yy, col) in ((6, (226, 84, 72, 255)), (10, (240, 120, 100, 255)),
                      (14, (196, 60, 56, 255))):
        pts = [(x, yy + 2.0 * math.sin(x / 5.0 + yy)) for x in range(33)]
        d.line(pts, fill=col, width=2)
    # lightning bolt
    bolt = [(19, 16), (14, 25), (18, 25), (13, 31), (21, 22), (17, 22),
            (22, 16)]
    d.polygon(bolt, fill=(255, 214, 170, 255))
    for _ in range(7):  # twinkles
        x, y = rng.uniform(2, 29), rng.uniform(18, 29)
        d.line([x - 2, y, x + 2, y], fill=(255, 160, 140, 255), width=1)
        d.line([x, y - 2, x, y + 2], fill=(255, 160, 140, 255), width=1)
    return img

def f_push_items(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    col = (235, 235, 225, 255)
    d.ellipse([14, 14, 18, 18], fill=(190, 190, 180, 255))
    for (dx, dy) in ((0, -1), (0, 1), (-1, 0), (1, 0),
                     (-1, -1), (1, -1), (-1, 1), (1, 1)):
        x1, y1 = 16 + dx * 5, 16 + dy * 5
        x2, y2 = 16 + dx * 11, 16 + dy * 11
        d.line([x1, y1, x2, y2], fill=col, width=2)
        # arrowhead
        px, py = -dy, dx
        d.polygon([(x2, y2),
                   (x2 - dx * 4 + px * 2, y2 - dy * 4 + py * 2),
                   (x2 - dx * 4 - px * 2, y2 - dy * 4 - py * 2)], fill=col)
    return img

def f_last_known(rng):
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    pale = (196, 222, 226, 255)
    faint = (140, 168, 174, 255)
    # dashed ring
    for k in range(12):
        a0 = math.radians(k * 30)
        a1 = math.radians(k * 30 + 16)
        d.line([16 + 12 * math.cos(a0), 16 + 12 * math.sin(a0),
                16 + 12 * math.cos(a1), 16 + 12 * math.sin(a1)],
               fill=faint, width=1)
    # ghost pin
    d.ellipse([11, 5, 21, 15], outline=pale, width=2)
    d.polygon([(12, 12), (20, 12), (16, 21)], outline=pale, width=2)
    d.ellipse([14, 8, 18, 12], fill=pale)
    # fading trail dots
    for (x, y, r) in ((7, 24, 2), (11, 26, 1), (16, 27, 1)):
        d.ellipse([x - r, y - r, x + r, y + r], fill=faint)
    return img

PHYSICAL = {
    "fd_last_known": f_last_known,
    "fd_migo_atmosphere": f_migo_atmosphere,
    "fd_nauseating_smell": f_nauseating_smell,
    "fd_push_items": f_push_items,
    "fd_scarlet_atmosphere": f_scarlet_atmosphere,
    "fd_scarlet_fog": f_scarlet_fog,
    "fd_scarlet_sea_gas": f_scarlet_sea_gas,
    "fd_scarlet_sea_wave": f_scarlet_sea_wave,
    "fd_scarlet_storm_lightning": f_scarlet_storm_lightning,
    "fd_shadow": f_shadow,
}

IDS = sorted(list(ZONES) + list(PHYSICAL))

def draw_one(fid):
    rng = random.Random(zlib.crc32(fid.encode()))
    if fid in ZONES:
        accent, picto = ZONES[fid]
        img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
        d = ImageDraw.Draw(img)
        brackets(d, accent)
        picto(d, accent, rng)
    else:
        img = PHYSICAL[fid](rng)
    alphas = set(img.getchannel("A").getdata())
    assert alphas <= {0, 255}, (fid, alphas)
    return img

def main():
    os.makedirs(BASE, exist_ok=True)
    imgs = {}
    for fid in IDS:
        img = draw_one(fid)
        imgs[fid] = img
        folder = os.path.join(BASE, fid)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, fid + ".png"))
        entry = [{"id": fid, "fg": fid, "bg": ""}]
        with open(os.path.join(folder, fid + ".json"), "w") as f:
            f.write(json.dumps(entry, indent=2) + "\n")
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS)
    # contact-sheet preview: 9 x 5 at 3x on dark ground
    cols, rows, cell = 9, 5, 96
    sheet = Image.new("RGBA", (cols * cell, rows * cell), (34, 34, 40, 255))
    for i, fid in enumerate(IDS):
        big = imgs[fid].resize((96, 96), Image.NEAREST)
        sheet.alpha_composite(big, ((i % cols) * cell, (i // cols) * cell))
    sheet.save(os.path.join(os.path.dirname(BASE), "..",
                            "preview_field2.png"))
    print("wrote", len(IDS), "field sprites + preview_field2.png")

if __name__ == "__main__":
    main()
