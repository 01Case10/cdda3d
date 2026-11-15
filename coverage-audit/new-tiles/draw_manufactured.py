#!/usr/bin/env python3
"""
Draw the missing manufactured / mechanisms / sunken-helicopter terrain
sprites from their game descriptions.

Sources (fork-src/cdda/data/json/furniture_and_terrain/, CDDA 6a3e59e0) --
the 18 ids below are the entries of terrain-manufactured.json,
terrain-mechanisms.json and special_use/sunken_helicopter_furniture.json
that are missing from UltiCa in the shipped universe
(coverage-audit/missing.json), with no folder yet under new-tiles/:

  t_missile                     "A section of an ICBM, an Intercontinental
                                 Ballistic Missile."  symbol #, light_blue
  t_missile_exploded            "blown-out missile... This isn't the kind
                                 of rocket that's going anywhere."
                                 symbol #, light_gray
  t_radiosphere_radio_tower_liminal_light
                                 copy-from t_radio_tower ("The structure
                                 of a radio transmission tower", symbol &,
                                 light_gray) + "there's a bright spotlight
                                 hanging off it", light_emitted 50
  t_nanofab_body                "A great column of advanced machinery...
                                 several 3D printers work in tandem with a
                                 robotic assembler"  symbol %, dark_gray
  t_nanofab_body_broken         the same column, "broken beyond repair...
                                 nothing but a useless hunk of steel"
  t_milking_machine             "A machine used in the dairy industry to
                                 milk cows."  symbol %, light_gray
  t_leanto                      "pine lean-to... roofed with pine branches"
                                 symbol ;, brown, THIN_OBSTACLE
  t_brick_oven_base             "A flat assembly of carefully stacked
                                 bricks that is not quite an oven."
                                 symbol =, color yellow (fire brick)
  t_gates_boom_barrier          copy-from t_gates_control_concrete
                                 ("mechanical winch... open or close a
                                 nearby gate", symbol 6, cyan_red):
                                 "Controls for a boom barrier... used to
                                 raise or lower a nearby boom barrier."
  t_intercom_balthazar          "A rudimentary intercom panel... set up in
                                 a hurry with the microphone and speaker
                                 just dangling."  symbol =, light_gray
  t_elevator_control_no_roof    copy-from t_elevator_control ("The control
                                 face for an elevator.  You could press the
                                 appropriate button...", symbol 6->.,
                                 dark_gray), roof t_null
  t_elevator_control_off_no_roof
                                 copy-from t_elevator_control_off ("It's
                                 currently unpowered."), roof t_null
  t_nuclear_reactor             "A naval A1B reactor, used in Gerald R.
                                 Ford-class aircraft carriers."
                                 symbol R, light_green
  t_sunkenhelicopter_floor      "waterlogged aisle... degraded after some
                                 time spent underwater" -- copy-from
                                 t_scrap_floor (crudely welded metal floor,
                                 cyan) + LIQUID/SWIMMABLE/DEEP_WATER
  t_sunkenhelicopter_floor_rotator
                                 the same aisle floor, at the rotating
                                 seat mount (name from the mapgen piece)
  t_sunkenhelicopter_windshield_n
                                 "A waterlogged sheet of glass that lets
                                 you see outside the vehicle" -- copy-from
                                 t_window (light_cyan)
  t_sunkenhelicopter_roof       "A waterlogged metal roof.  Absolutely
                                 unrecoverable."
  t_sunkenhelicopter_rotor      "A set of four military-grade helicopter
                                 rotor blades... degraded after some time
                                 spent underwater."

Design: UltiCa decomposed layout, all sprites 32x32 on the normal sheet.
Full-tile sprites (the two waterlogged floors, the roof, the brick oven
base) are fg with bg "" and are FULLY OPAQUE -- translucent-looking
marks (water sheen) are composited with Image.alpha_composite overlays,
never drawn as semi-transparent pixels.  Machines and mechanisms are
transparent fg with bg ""; the rotor lies in the flooded cabin, so it
sits over bg "t_sunkenhelicopter_floor".  Flat fills, dark outlines,
one highlight pass -- UltiCa style.  Deterministic: every random choice
comes from a per-sprite seeded Random.
Output: pngs_normal_32x32/terrain/<id>/ (png + tile-entry json + credits).
"""
from PIL import Image, ImageDraw
import json, math, os, random

BASE = os.path.dirname(os.path.abspath(__file__))
S = 32

def new_img():
    return Image.new("RGBA", (S, S), (0, 0, 0, 0))

def full(color):
    return Image.new("RGBA", (S, S), color)

def overlay(img, draw_fn):
    """Composite a translucent overlay onto img (alpha_composite, never
    direct semi-transparent drawing onto the base)."""
    ov = new_img()
    d = ImageDraw.Draw(ov)
    draw_fn(d)
    return Image.alpha_composite(img, ov)

# ---------------------------------------------------------------- missile
def _missile_body(d, base, shade, hi, edge):
    # a thick vertical ICBM body section, slightly domed cut at the top
    d.rectangle([9, 6, 23, 31], fill=base)
    d.pieslice([9, 1, 23, 11], start=180, end=360, fill=base)
    d.rectangle([9, 6, 12, 31], fill=shade)          # left shade
    d.rectangle([20, 6, 23, 31], fill=shade)         # right shade
    d.rectangle([14, 4, 16, 31], fill=hi)            # center highlight
    d.line([9, 6, 9, 31], fill=edge, width=1)
    d.line([23, 6, 23, 31], fill=edge, width=1)
    # stub fins at the base
    d.polygon([(9, 31), (5, 31), (9, 25)], fill=shade, outline=edge)
    d.polygon([(23, 31), (27, 31), (23, 25)], fill=shade, outline=edge)

def draw_missile():
    rng = random.Random(3301)
    img = new_img(); d = ImageDraw.Draw(img)
    _missile_body(d, (188, 210, 228, 255), (150, 176, 198, 255),
                  (226, 240, 248, 255), (96, 122, 144, 255))
    # panel bands + rivets
    for y in (11, 21):
        d.line([9, y, 23, y], fill=(110, 138, 158, 255), width=1)
        for x in range(11, 23, 3):
            d.point((x, y - 1), fill=(110, 138, 158, 255))
    # red warning band with white stencil ticks
    d.rectangle([9, 14, 23, 17], fill=(178, 52, 48, 255))
    for x in (11, 14, 17, 20):
        d.rectangle([x, 15, x + 1, 16], fill=(240, 240, 240, 255))
    # scuffs
    for _ in range(7):
        x = rng.uniform(10, 22); y = rng.uniform(19, 30)
        d.point((x, y), fill=(140, 164, 184, 255))
    return img

def draw_missile_exploded():
    rng = random.Random(3302)
    img = new_img(); d = ImageDraw.Draw(img)
    _missile_body(d, (172, 172, 176, 255), (138, 138, 142, 255),
                  (204, 204, 208, 255), (84, 84, 88, 255))
    for y in (11, 21):
        d.line([9, y, 23, y], fill=(104, 104, 108, 255), width=1)
    # the burst: a jagged dark rupture across the upper body, metal
    # petals peeled outward
    d.polygon([(10, 8), (15, 12), (13, 16), (18, 19), (22, 15), (20, 11),
               (23, 9), (23, 18), (20, 22), (14, 21), (10, 18)],
              fill=(26, 26, 30, 255))
    for (pts, col) in [
            ([(10, 8), (5, 4), (9, 13)], (150, 150, 154, 255)),
            ([(23, 9), (28, 5), (24, 14)], (150, 150, 154, 255)),
            ([(13, 20), (9, 25), (16, 22)], (142, 142, 146, 255))]:
        d.polygon(pts, fill=col, outline=(84, 84, 88, 255))
    # scorch blotches + cracks below the rupture
    for _ in range(9):
        x = rng.uniform(10, 21); y = rng.uniform(19, 29)
        w = rng.uniform(2, 4.5)
        d.ellipse([x, y, x + w, y + w * 0.7], fill=(44, 42, 40, 255))
    d.line([14, 22, 12, 27], fill=(84, 84, 88, 255), width=1)
    d.line([19, 22, 21, 26], fill=(84, 84, 88, 255), width=1)
    # one panel hanging loose at the base
    d.polygon([(17, 27), (24, 29), (23, 31), (16, 30)],
              fill=(156, 156, 160, 255), outline=(84, 84, 88, 255))
    return img

# ---------------------------------------------------------------- radio tower + spotlight
def draw_radio_tower_spotlight():
    img = new_img(); d = ImageDraw.Draw(img)
    steel, dark = (142, 146, 150, 255), (104, 108, 112, 255)
    # lattice mast: chords taper toward the top, X-braced
    for t in range(9):
        y0 = 31 - t * 3.6; y1 = 31 - (t + 1) * 3.6
        lx0 = 6 + t * 0.45; rx0 = 21 - t * 0.45
        lx1 = 6 + (t + 1) * 0.45; rx1 = 21 - (t + 1) * 0.45
        d.line([lx0, y0, rx1, y1], fill=dark, width=1)
        d.line([rx0, y0, lx1, y1], fill=dark, width=1)
        d.line([lx0, y0, rx0, y0], fill=dark, width=1)
    d.line([6, 31, 9.6, 2], fill=steel, width=2)
    d.line([21, 31, 17.4, 2], fill=steel, width=2)
    d.line([8, 31, 5, 31], fill=dark, width=2)   # feet
    d.line([19, 31, 22, 31], fill=dark, width=2)
    # crossarm near the top, spotlight hanging off its end
    d.line([13, 6, 28, 6], fill=steel, width=2)
    d.line([24, 6, 24, 9], fill=dark, width=1)   # drop bracket
    # glow first (composited), then the lamp itself on top
    def glow(gd):
        gd.ellipse([17, 5, 31, 19], fill=(255, 244, 160, 56))
        gd.ellipse([20, 7, 29, 16], fill=(255, 246, 180, 60))
    img = overlay(img, glow)
    d = ImageDraw.Draw(img)
    d.polygon([(21, 9), (28, 9), (27, 14), (22, 14)],
              fill=(58, 60, 64, 255), outline=(34, 36, 40, 255))
    d.ellipse([22.4, 10.2, 26.6, 13.8], fill=(252, 242, 158, 255))
    d.ellipse([23.6, 11.2, 25.4, 12.8], fill=(255, 255, 240, 255))
    for (x, y) in [(19, 8), (29, 12), (21, 16)]:  # stray glow flecks
        d.point((x, y), fill=(250, 240, 170, 255))
    return img

# ---------------------------------------------------------------- nanofab
def _nanofab_frame(d):
    body, shade, hi, edge = (76, 78, 84, 255), (60, 62, 68, 255), \
        (104, 106, 114, 255), (40, 42, 48, 255)
    d.rectangle([7, 3, 25, 31], fill=body)
    d.rectangle([7, 3, 10, 31], fill=shade)
    d.line([24, 3, 24, 31], fill=hi, width=1)
    d.rectangle([7, 3, 25, 6], fill=(94, 96, 104, 255))      # top cap
    for x in range(10, 24, 3):                                # cap vents
        d.line([x, 4, x + 1, 4], fill=edge, width=1)
    d.rectangle([6, 29, 26, 31], fill=shade)                  # plinth
    d.line([7, 3, 25, 3], fill=edge, width=1)
    d.line([7, 3, 7, 31], fill=edge, width=1)
    d.line([25, 3, 25, 31], fill=edge, width=1)
    # cables snaking out of the base
    d.line([(7, 29), (3, 30), (2, 28)], fill=edge, width=2)
    d.line([(25, 29), (29, 30), (30, 27)], fill=edge, width=2)
    return edge

def draw_nanofab():
    rng = random.Random(3304)
    img = new_img(); d = ImageDraw.Draw(img)
    edge = _nanofab_frame(d)
    teal = (72, 208, 196, 255)
    # print chamber: dark window, a half-printed pale object inside
    d.rectangle([11, 10, 21, 21], fill=(22, 28, 32, 255), outline=teal)
    d.rectangle([13, 17, 19, 20], fill=(120, 128, 134, 255))  # build plate
    d.rectangle([14, 13, 18, 17], fill=(172, 182, 188, 255))  # printed block
    d.rectangle([14, 13, 18, 14], fill=(206, 216, 220, 255))
    d.line([16, 10, 16, 13], fill=(150, 158, 164, 255), width=1)  # print head
    # control screen with teal readouts
    d.rectangle([12, 23, 20, 27], fill=(16, 34, 34, 255), outline=edge)
    for i, wdt in enumerate((5, 3, 4)):
        d.line([13, 24 + i, 13 + wdt, 24 + i], fill=teal, width=1)
    # indicator lights
    d.point((22, 12), fill=(92, 236, 178, 255))
    d.point((22, 15), fill=(250, 190, 80, 255))
    d.point((22, 18), fill=(92, 236, 178, 255))
    for _ in range(5):  # faint panel wear
        x = rng.uniform(8, 24); y = rng.uniform(24, 30)
        d.point((x, y), fill=(66, 68, 74, 255))
    return img

def draw_nanofab_broken():
    rng = random.Random(3305)
    img = new_img(); d = ImageDraw.Draw(img)
    edge = _nanofab_frame(d)
    # dead chamber: dark, cracked glass, slumped debris inside
    d.rectangle([11, 10, 21, 21], fill=(14, 16, 18, 255),
                outline=(52, 54, 60, 255))
    d.rectangle([13, 18, 19, 20], fill=(70, 72, 76, 255))
    d.polygon([(14, 18), (16, 15), (18, 18)], fill=(88, 90, 94, 255))
    for pts in [[(12, 11), (15, 15), (14, 19)], [(15, 15), (19, 13)],
                [(15, 15), (18, 18), (20, 20)]]:  # cracks
        d.line(pts, fill=(148, 152, 156, 255), width=1)
    # screen black and cracked too
    d.rectangle([12, 23, 20, 27], fill=(10, 10, 12, 255), outline=edge)
    d.line([13, 26, 19, 23], fill=(120, 124, 128, 255), width=1)
    # side panel hanging open, askew
    d.polygon([(25, 12), (30, 14), (29, 23), (25, 21)],
              fill=(66, 68, 74, 255), outline=edge)
    d.rectangle([25, 13, 25, 20], fill=(16, 16, 18, 255))  # the dark gap
    # scorch at the base + debris + one severed cable end
    d.ellipse([9, 26, 16, 31], fill=(28, 26, 24, 255))
    for _ in range(10):
        x = rng.uniform(4, 28); y = rng.uniform(28, 31)
        d.point((x, y), fill=(96, 98, 102, 255))
    d.line([(25, 29), (28, 28)], fill=(200, 120, 60, 255), width=1)
    d.point((22, 12), fill=(120, 40, 36, 255))  # one dying red dot
    return img

# ---------------------------------------------------------------- milking machine
def draw_milking_machine():
    img = new_img(); d = ImageDraw.Draw(img)
    steel, shade, hi, edge = (198, 200, 202, 255), (164, 166, 170, 255), \
        (232, 234, 236, 255), (96, 98, 102, 255)
    hose = (72, 74, 78, 255)
    # hoses first (behind the vessels): cluster bar up to the teat cups,
    # and a coil on the floor
    d.line([(14, 12), (10, 20), (11, 27)], fill=hose, width=2)
    d.line([(18, 12), (24, 18), (25, 24)], fill=hose, width=2)
    d.arc([18, 24, 30, 31], start=0, end=300, fill=hose, width=2)
    # milk canister (left): cylinder + lid + handle
    d.rectangle([7, 13, 18, 30], fill=steel)
    d.rectangle([7, 13, 9, 30], fill=shade)
    d.rectangle([12, 13, 13, 30], fill=hi)
    d.ellipse([7, 10, 18, 15], fill=(178, 180, 184, 255), outline=edge)
    d.arc([9, 7, 16, 12], start=180, end=360, fill=edge, width=1)
    d.line([7, 30, 18, 30], fill=edge, width=1)
    # pump box (right) with a round gauge
    d.rectangle([20, 17, 29, 30], fill=(176, 178, 182, 255), outline=edge)
    d.ellipse([22, 19, 27, 24], fill=(240, 240, 238, 255), outline=edge)
    d.line([24.5, 21.5, 26, 20], fill=(60, 60, 64, 255), width=1)
    d.rectangle([22, 26, 24, 28], fill=(188, 60, 52, 255))  # switch
    # teat-cup cluster hanging from a bar above the canister
    d.line([9, 8, 21, 8], fill=edge, width=2)
    for x in (11, 14, 17, 20):
        d.line([x, 8, x, 10], fill=hose, width=1)
        d.polygon([(x - 1.6, 10), (x + 1.6, 10), (x + 0.9, 14),
                   (x - 0.9, 14)], fill=(104, 106, 110, 255), outline=edge)
    return img

# ---------------------------------------------------------------- lean-to
def draw_leanto():
    rng = random.Random(3307)
    img = new_img(); d = ImageDraw.Draw(img)
    pole, pole_hi = (104, 76, 48, 255), (134, 100, 64, 255)
    # sheltered shadow under the slope
    d.polygon([(5, 14), (27, 24), (27, 31), (5, 31)], fill=(44, 38, 28, 255))
    # support poles: tall back-left, short front-right, one mid prop
    d.line([5, 5, 5, 31], fill=pole, width=3)
    d.line([4, 5, 4, 31], fill=pole_hi, width=1)
    d.line([27, 19, 27, 31], fill=pole, width=3)
    d.line([26, 19, 26, 31], fill=pole_hi, width=1)
    d.line([16, 13, 16, 31], fill=pole, width=2)
    # the sloped bough roof: layered pine-branch strokes along the slope
    greens = [(52, 84, 38, 255), (68, 104, 48, 255), (84, 118, 56, 255),
              (44, 70, 32, 255)]
    for row in range(6):
        t = row / 5.0
        y0 = 5 + t * 13           # slope from (3,5) down to (29,18)
        for _ in range(15):
            x = rng.uniform(2, 28)
            y = y0 + (x - 3) * (13 / 26.0) * 0.32 + rng.uniform(-1.4, 1.4)
            col = greens[rng.randrange(len(greens))]
            ln = rng.uniform(2.4, 4.2)
            d.line([x, y, x + ln * 0.8, y + ln * 0.55], fill=col, width=1)
            d.line([x, y, x + ln * 0.3, y + ln * 0.8], fill=col, width=1)
    # branch sticks poking through the thatch
    for (x, y) in [(8, 6), (18, 10), (25, 15)]:
        d.line([x, y, x + 3, y + 2], fill=(96, 70, 44, 255), width=1)
    # a few fallen needles on the ground in front
    for _ in range(8):
        x = rng.uniform(6, 28); y = rng.uniform(27, 31)
        d.line([x, y, x + 1.6, y + 0.6], fill=(70, 96, 46, 255), width=1)
    return img

# ---------------------------------------------------------------- brick oven base
def draw_brick_oven_base():
    rng = random.Random(3308)
    img = full((148, 134, 114, 255))  # mortar
    d = ImageDraw.Draw(img)
    bricks = [(208, 172, 98, 255), (198, 160, 86, 255), (218, 184, 110, 255),
              (190, 150, 78, 255)]
    bh, bw = 5, 10
    for row in range(6):
        y = 1 + row * bh
        off = (bw // 2) if row % 2 else 0
        for x in range(-1, 4):
            bx = x * bw + off
            col = bricks[rng.randrange(len(bricks))]
            d.rectangle([bx + 1, y, bx + bw - 1, y + bh - 1], fill=col)
            d.line([bx + 1, y, bx + bw - 1, y],
                   fill=(228, 198, 128, 255), width=1)  # top highlight
    # the stacked front edge: a darker course along the bottom
    d.rectangle([0, 28, 31, 31], fill=(168, 136, 72, 255))
    for x in range(0, 32, 6):
        d.line([x, 28, x, 31], fill=(120, 100, 66, 255), width=1)
    d.line([0, 28, 31, 28], fill=(104, 88, 58, 255), width=1)
    for _ in range(16):  # kiln dust specks
        x = rng.uniform(0, 31); y = rng.uniform(0, 27)
        d.point((x, y), fill=(150, 124, 78, 255))
    return img

# ---------------------------------------------------------------- boom barrier controls
def draw_boom_barrier():
    img = new_img(); d = ImageDraw.Draw(img)
    conc, conc_d, edge = (170, 170, 166, 255), (138, 138, 134, 255), \
        (92, 92, 90, 255)
    # the striped boom arm first (behind the pedestal), raised diagonally
    for i in range(6):
        t0, t1 = i / 6.0, (i + 1) / 6.0
        x0, y0 = 14 + 17 * t0, 13 - 10 * t0
        x1, y1 = 14 + 17 * t1, 13 - 10 * t1
        col = (206, 52, 46, 255) if i % 2 == 0 else (234, 234, 234, 255)
        d.line([x0, y0, x1, y1], fill=col, width=4)
    d.line([14, 13, 31, 3], fill=edge, width=1)
    # concrete pedestal
    d.rectangle([6, 14, 16, 31], fill=conc)
    d.rectangle([6, 14, 8, 31], fill=conc_d)
    d.rectangle([5, 29, 17, 31], fill=conc_d)
    d.line([6, 14, 16, 14], fill=(196, 196, 192, 255), width=1)
    # control box on top: cyan steel, red button (the parent's cyan_red)
    d.rectangle([7, 8, 15, 14], fill=(104, 168, 178, 255), outline=edge)
    d.ellipse([9.4, 9.6, 12.6, 12.8], fill=(206, 52, 46, 255),
              outline=(120, 30, 26, 255))
    # winch drum on the pedestal face + crank handle
    d.ellipse([8, 19, 15, 26], fill=(118, 126, 134, 255), outline=edge)
    d.ellipse([10, 21, 13, 24], fill=(84, 90, 98, 255))
    d.line([15, 22, 20, 19], fill=edge, width=2)   # crank arm
    d.ellipse([19, 17.4, 22, 20.4], fill=(60, 60, 62, 255))  # crank knob
    # cable from the drum up to the boom pivot
    d.line([13, 19, 15, 13], fill=(70, 70, 72, 255), width=1)
    return img

# ---------------------------------------------------------------- intercom
def draw_intercom():
    img = new_img(); d = ImageDraw.Draw(img)
    plate, shade, edge = (188, 190, 192, 255), (158, 160, 164, 255), \
        (104, 106, 110, 255)
    # the panel plate, slightly proud of the wall
    d.rectangle([9, 3, 23, 25], fill=plate, outline=edge)
    d.rectangle([9, 3, 11, 25], fill=shade)
    for (x, y) in [(11, 5), (21, 5), (11, 23)]:  # one screw missing (hurry!)
        d.point((x, y), fill=(80, 82, 86, 255))
    d.point((21, 23), fill=(30, 30, 32, 255))     # empty screw hole
    # speaker grille: dot grid
    for gy in range(7, 14, 2):
        for gx in range(13, 21, 2):
            d.point((gx, gy), fill=(70, 72, 76, 255))
    # call button + label slit
    d.ellipse([13, 17, 17, 21], fill=(128, 130, 134, 255), outline=edge)
    d.point((14, 18), fill=(226, 228, 230, 255))
    d.rectangle([18, 18, 21, 20], fill=(168, 170, 174, 255), outline=edge)
    # the dangling microphone: curly cord off the panel's bottom edge,
    # mic hanging loose at the end of it
    pts = []
    for i in range(21):
        t = i / 20.0
        pts.append((13 + 1.8 * math.sin(t * 12.0), 25 + t * 4.5))
    d.line(pts, fill=(70, 72, 76, 255), width=1)
    d.rounded_rectangle([9, 28, 15, 31], radius=1, fill=(66, 68, 72, 255),
                        outline=(40, 42, 46, 255))
    d.line([10, 29, 14, 29], fill=(120, 122, 126, 255), width=1)
    # a loose wire loop hanging off the right side
    d.arc([19, 24, 27, 31], start=300, end=160, fill=(90, 92, 96, 255),
          width=1)
    return img

# ---------------------------------------------------------------- elevator controls
def _elevator_control(off):
    img = new_img(); d = ImageDraw.Draw(img)
    face, shade, edge = (116, 118, 124, 255), (88, 90, 96, 255), \
        (52, 54, 60, 255)
    # wall plate with a rounded top
    d.rounded_rectangle([10, 3, 22, 29], radius=3, fill=face, outline=edge)
    d.rectangle([10, 6, 12, 26], fill=shade)
    d.rectangle([9, 29, 23, 31], fill=(70, 72, 78, 255))  # plinth shadow
    # floor display window
    if off:
        d.rectangle([13, 6, 19, 10], fill=(26, 28, 32, 255), outline=edge)
        btn = (96, 98, 102, 255)
    else:
        d.rectangle([13, 6, 19, 10], fill=(168, 216, 242, 255), outline=edge)
        d.line([16, 7, 16, 9], fill=(40, 80, 110, 255), width=1)  # digit "1"
        d.point((15, 9), fill=(40, 80, 110, 255))
        btn = (150, 156, 162, 255)
    # up / down buttons
    for (cy, up) in [(15, True), (22, False)]:
        d.ellipse([13, cy - 3, 19, cy + 3], fill=btn, outline=edge)
        if up:
            d.polygon([(16, cy - 1.6), (14.6, cy + 1), (17.4, cy + 1)],
                      fill=edge)
        else:
            d.polygon([(16, cy + 1.6), (14.6, cy - 1), (17.4, cy - 1)],
                      fill=edge)
    if not off:  # the up button's call lamp is lit
        d.ellipse([13, 12, 19, 18], outline=(168, 216, 242, 255))
    for (x, y) in [(12, 5), (20, 5), (12, 27), (20, 27)]:
        d.point((x, y), fill=(60, 62, 68, 255))
    return img

def draw_elevator_control_no_roof():
    return _elevator_control(False)

def draw_elevator_control_off_no_roof():
    return _elevator_control(True)

# ---------------------------------------------------------------- nuclear reactor
def draw_nuclear_reactor():
    img = new_img(); d = ImageDraw.Draw(img)
    body, shade, hi, edge = (170, 198, 168, 255), (138, 168, 136, 255), \
        (204, 226, 198, 255), (78, 102, 76, 255)
    # pressure vessel: domed top, cylindrical body, skirt base
    d.pieslice([5, 1, 27, 13], start=180, end=360, fill=hi, outline=edge)
    d.rectangle([5, 7, 27, 28], fill=body)
    d.rectangle([5, 7, 9, 28], fill=shade)
    d.rectangle([14, 7, 17, 28], fill=hi)
    d.line([27, 7, 27, 28], fill=edge, width=1)
    d.rectangle([16, 0, 19, 3], fill=shade, outline=edge)  # lifting lug
    # bolted flange ring
    d.rectangle([5, 11, 27, 14], fill=shade)
    for x in range(7, 27, 3):
        d.point((x, 12), fill=(52, 70, 50, 255))
        d.point((x, 13), fill=(52, 70, 50, 255))
    # radiation trefoil on a yellow disc
    d.ellipse([12, 17, 21, 26], fill=(234, 206, 84, 255), outline=edge)
    d.ellipse([15.6, 20.6, 17.4, 22.4], fill=(52, 58, 50, 255))
    for (cx, cy) in [(16.5, 19.2), (14.9, 22.9), (18.1, 22.9)]:
        d.ellipse([cx - 1.3, cy - 1.3, cx + 1.3, cy + 1.3],
                  fill=(52, 58, 50, 255))
    # pipes out of the right side and base
    d.rectangle([27, 18, 31, 21], fill=shade, outline=edge)
    d.rectangle([27, 24, 30, 27], fill=shade, outline=edge)
    d.rectangle([4, 28, 28, 31], fill=(120, 148, 118, 255))
    d.line([4, 28, 28, 28], fill=edge, width=1)
    return img

# ---------------------------------------------------------------- sunken helicopter
def _water_sheen(img, rng, streaks=5):
    def sheen(d):
        for _ in range(streaks):
            x = rng.uniform(2, 24); y = rng.uniform(2, 28)
            ln = rng.uniform(4, 9)
            d.line([x, y, x + ln, y - 1], fill=(190, 235, 228, 52), width=1)
        for _ in range(6):
            x = rng.uniform(2, 29); y = rng.uniform(2, 29)
            d.ellipse([x, y, x + 1.6, y + 1.6], outline=(200, 240, 234, 70))
    return overlay(img, sheen)

def draw_sunken_floor():
    rng = random.Random(3313)
    img = draw_sunken_floor_base(rng)  # deep water over the aisle
    d = ImageDraw.Draw(img)
    for _ in range(14):  # silt motes
        x = rng.uniform(0, 31); y = rng.uniform(0, 31)
        d.point((x, y), fill=(52, 108, 106, 255))
    return _water_sheen(img, rng)

def draw_sunken_floor_rotator():
    rng = random.Random(3314)
    img = draw_sunken_floor_base(rng)
    d = ImageDraw.Draw(img)
    # the rotating seat mount: a bolted circular turntable under the water
    d.ellipse([7, 7, 25, 25], fill=(30, 62, 64, 255))
    d.ellipse([8, 8, 24, 24], outline=(96, 116, 118, 255), width=2)
    d.ellipse([11, 11, 21, 21], fill=(52, 76, 78, 255),
              outline=(24, 50, 52, 255))
    for a in range(8):
        ang = a * math.tau / 8
        x = 16 + 7.6 * math.cos(ang); y = 16 + 7.6 * math.sin(ang)
        d.ellipse([x - 0.9, y - 0.9, x + 0.9, y + 0.9],
                  fill=(150, 160, 160, 255))
    d.ellipse([14.4, 14.4, 17.6, 17.6], fill=(140, 150, 150, 255),
              outline=(24, 50, 52, 255))
    return _water_sheen(img, rng, streaks=3)

def draw_sunken_floor_base(rng):
    img = full((24, 70, 74, 255))
    d = ImageDraw.Draw(img)
    for (x0, y0, x1, y1) in [(0, 0, 15, 15), (16, 0, 31, 15),
                             (0, 16, 15, 31), (16, 16, 31, 31)]:
        d.rectangle([x0, y0, x1, y1], outline=(14, 46, 50, 255))
        d.rectangle([x0 + 1, y0 + 1, x1 - 1, y1 - 1],
                    outline=(38, 92, 96, 255))
    for _ in range(10):
        x = rng.uniform(0, 30); y = rng.uniform(0, 30)
        d.line([x, y, x + rng.uniform(-1, 2), y + rng.uniform(1, 4)],
               fill=(104, 76, 44, 255), width=1)
    return img

def draw_sunken_roof():
    rng = random.Random(3315)
    img = full((72, 82, 84, 255))  # water-stained metal sheet
    d = ImageDraw.Draw(img)
    # panel bands with rivet rows
    for y in (10, 21):
        d.line([0, y, 31, y], fill=(48, 56, 58, 255), width=1)
        d.line([0, y + 1, 31, y + 1], fill=(104, 116, 118, 255), width=1)
        for x in range(2, 31, 4):
            d.point((x, y - 2), fill=(46, 54, 56, 255))
            d.point((x, y - 3), fill=(118, 130, 132, 255))
    # degradation: algae-tinted patches and rust weep
    for _ in range(9):
        x = rng.uniform(0, 26); y = rng.uniform(0, 26)
        w = rng.uniform(3, 7); h = rng.uniform(2, 4)
        d.ellipse([x, y, x + w, y + h], fill=(80, 102, 70, 255))
    for _ in range(7):
        x = rng.uniform(0, 30); y = rng.uniform(0, 24)
        d.line([x, y, x, y + rng.uniform(3, 6)], fill=(110, 82, 50, 255),
               width=1)
    # one bent corner, lifted and bright-edged
    d.polygon([(24, 0), (31, 0), (31, 6)], fill=(52, 60, 62, 255))
    d.line([24, 0, 31, 6], fill=(150, 162, 164, 255), width=1)
    # shallow water sheen pooling on the sheet
    return _water_sheen(img, rng, streaks=4)

def draw_sunken_windshield():
    rng = random.Random(3316)
    img = new_img(); d = ImageDraw.Draw(img)
    frame, edge = (66, 74, 76, 255), (38, 44, 46, 255)
    # the framed sheet of glass
    d.rounded_rectangle([2, 7, 30, 26], radius=3, fill=frame, outline=edge)
    d.rounded_rectangle([5, 10, 27, 23], radius=2,
                        fill=(100, 142, 138, 255), outline=edge)
    # algae smears clouding the glass
    for _ in range(8):
        x = rng.uniform(6, 24); y = rng.uniform(11, 20)
        w = rng.uniform(2.5, 5.5); h = rng.uniform(1.5, 3)
        d.ellipse([x, y, x + w, y + h], fill=(84, 116, 80, 255))
    # cracks
    d.line([8, 11, 13, 16, 12, 21], fill=(206, 226, 220, 255), width=1)
    d.line([13, 16, 18, 15], fill=(206, 226, 220, 255), width=1)
    d.line([24, 12, 21, 17], fill=(206, 226, 220, 255), width=1)
    # rivets on the frame
    for (x, y) in [(4, 9), (28, 9), (4, 24), (28, 24), (16, 8), (16, 25)]:
        d.point((x, y), fill=(140, 150, 150, 255))
    # a murky sheen across the pane (composited, not drawn raw)
    def sheen(dd):
        dd.polygon([(7, 22), (15, 10), (19, 10), (11, 22)],
                   fill=(220, 245, 240, 44))
    img = overlay(img, sheen)
    return img

def draw_sunken_rotor():
    rng = random.Random(3317)
    img = new_img(); d = ImageDraw.Draw(img)
    blade, shade, edge = (88, 96, 90, 255), (62, 70, 64, 255), \
        (36, 42, 38, 255)
    cx = cy = 16
    # four blades: two long diagonals, one bent, one snapped short
    def blade_poly(ang, ln, w0, w1, kink=0.0):
        dx, dy = math.cos(ang), math.sin(ang)
        px, py = -dy, dx
        mx, my = cx + dx * ln * 0.55, cy + dy * ln * 0.55 + kink
        ex, ey = cx + dx * ln, cy + dy * ln + kink * 1.7
        return [(cx + px * w0, cy + py * w0), (mx + px * w1, my + py * w1),
                (ex, ey), (mx - px * w1, my - py * w1),
                (cx - px * w0, cy - py * w0)]
    for (ang, ln, kink) in [(2.55, 15.5, 0.0), (-0.55, 15.5, 0.0),
                            (0.85, 15.0, 2.2)]:
        d.polygon(blade_poly(ang, ln, 2.4, 1.5, kink), fill=blade,
                  outline=edge)
    # the snapped blade: short, jagged tip
    pts = blade_poly(-2.25, 8.5, 2.4, 1.6)
    d.polygon(pts, fill=blade, outline=edge)
    ex, ey = pts[2]
    d.line([ex - 1, ey - 1, ex + 1.4, ey + 1], fill=(150, 156, 148, 255),
           width=1)
    # rust and algae degradation along the blades
    for _ in range(16):
        a = rng.uniform(0, math.tau); r = rng.uniform(4, 14)
        x = cx + r * math.cos(a); y = cy + r * math.sin(a)
        col = (118, 88, 58, 255) if rng.random() < 0.5 else (76, 104, 66, 255)
        d.point((x, y), fill=col)
    # hub
    d.ellipse([12, 12, 20, 20], fill=shade, outline=edge)
    d.ellipse([14.2, 14.2, 17.8, 17.8], fill=(120, 128, 120, 255),
              outline=edge)
    return img

# ---------------------------------------------------------------- jobs
# (id, image, fg sprite, bg)
JOBS = [
    ("t_missile", draw_missile(), "t_missile", ""),
    ("t_missile_exploded", draw_missile_exploded(), "t_missile_exploded", ""),
    ("t_radiosphere_radio_tower_liminal_light", draw_radio_tower_spotlight(),
     "t_radiosphere_radio_tower_liminal_light", ""),
    ("t_nanofab_body", draw_nanofab(), "t_nanofab_body", ""),
    ("t_nanofab_body_broken", draw_nanofab_broken(),
     "t_nanofab_body_broken", ""),
    ("t_milking_machine", draw_milking_machine(), "t_milking_machine", ""),
    ("t_leanto", draw_leanto(), "t_leanto", ""),
    ("t_brick_oven_base", draw_brick_oven_base(), "t_brick_oven_base", ""),
    ("t_gates_boom_barrier", draw_boom_barrier(), "t_gates_boom_barrier", ""),
    ("t_intercom_balthazar", draw_intercom(), "t_intercom_balthazar", ""),
    ("t_elevator_control_no_roof", draw_elevator_control_no_roof(),
     "t_elevator_control_no_roof", ""),
    ("t_elevator_control_off_no_roof", draw_elevator_control_off_no_roof(),
     "t_elevator_control_off_no_roof", ""),
    ("t_nuclear_reactor", draw_nuclear_reactor(), "t_nuclear_reactor", ""),
    ("t_sunkenhelicopter_floor", draw_sunken_floor(),
     "t_sunkenhelicopter_floor", ""),
    ("t_sunkenhelicopter_floor_rotator", draw_sunken_floor_rotator(),
     "t_sunkenhelicopter_floor_rotator", ""),
    ("t_sunkenhelicopter_windshield_n", draw_sunken_windshield(),
     "t_sunkenhelicopter_windshield_n", ""),
    ("t_sunkenhelicopter_roof", draw_sunken_roof(),
     "t_sunkenhelicopter_roof", ""),
    ("t_sunkenhelicopter_rotor", draw_sunken_rotor(),
     "t_sunkenhelicopter_rotor", "t_sunkenhelicopter_floor"),
]

CREDITS = ("Drawn from the terrain's in-game description for the CDDA Voxel\n"
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.\n")

for tid, img, fg, bg in JOBS:
    folder = os.path.join(BASE, "pngs_normal_32x32", "terrain", tid)
    os.makedirs(folder, exist_ok=True)
    img.save(os.path.join(folder, tid + ".png"))
    entry = {"id": tid, "fg": fg, "bg": bg}
    with open(os.path.join(folder, tid + ".json"), "w") as f:
        json.dump([entry], f, indent=2)
        f.write("\n")
    with open(os.path.join(folder, "credits.txt"), "w") as f:
        f.write(CREDITS)
    print("wrote", tid)

# ---------------------------------------------------------------- preview
def tile_img(tid):
    p = os.path.join(BASE, "pngs_normal_32x32", "terrain", tid, tid + ".png")
    return Image.open(p).convert("RGBA")

BG_TILES = {bg for _tid, _img, _fg, bg in JOBS if bg}

def composited(tid, bg):
    """The tile as the game shows it: fg over its bg."""
    fg = tile_img(tid)
    if not bg:
        return fg
    out = tile_img(bg).copy()
    out.alpha_composite(fg)
    return out

CELL = 64
COLS = 6
ROWS = (len(JOBS) + COLS - 1) // COLS
sheet = Image.new("RGBA", (COLS * CELL, ROWS * CELL), (24, 24, 28, 255))
for i, (tid, _img, _fg, bg) in enumerate(JOBS):
    t = composited(tid, bg).resize((CELL, CELL), Image.NEAREST)
    sheet.alpha_composite(t, ((i % COLS) * CELL, (i // COLS) * CELL))
sheet.save(os.path.join(BASE, "preview_manufactured.png"))
print("wrote preview_manufactured.png", sheet.size)
