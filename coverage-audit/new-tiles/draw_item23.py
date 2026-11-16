#!/usr/bin/env python3
"""Batch ITEM23 (Phase 2g.4 scope extension): fake
items, final set -- the pseudo training gear and the
stationary appliance-tool stand-ins.

10 ids from data/json/items/fake.json (authoritative
list embedded below; source list:
~/workspace/item-batches/23.txt). Copy-from chains
resolved against the data before drawing: the five
tool stand-ins inherit the fake_item / fake_appliance_
tool base and differ only by name -- each is drawn as
the real stationary tool it stands in for.

The set, from the entries' names/descriptions:

- pseudo_magazine (MAGAZINE, red, symbol @): "Pseudo
  magazine for use in dirty vehicle hacks" -- a battery
  magazine with a million-charge capacity: a curved
  red box magazine with ribbed walls, feed lips, a
  baseplate, and a yellow charge bolt on its flank.
- pseudo_punching_bag (dark_gray, symbol 0): "heavy
  punching bag" -- a dark gray cylindrical heavy bag
  hanging from chains, with stitched seams and a
  reinforced bottom.
- pseudo_training_dummy_light (brown, symbol @):
  "training dummy" -- a straw-and-timber practice
  dummy: round head, crossbar arms, padded torso on a
  post stand.
- pseudo_training_dummy_heavy (brown, symbol @):
  "armored training dummy" -- the same dummy fitted
  with a steel helmet dome, riveted chest plate and
  arm guards.
- pseudo_treadmill_mechanical (dark_gray, symbol L):
  "gravity treadmill" -- a non-motorized treadmill in
  side view: inclined slatted belt deck, front uprights
  and a handlebar.
- router_tool (TOOL): "router" -- a woodworking
  router: cylindrical motor body, two knob handles,
  flat base plate and a bit protruding below.
- scutcheritem (TOOL): "stationary scutcher" -- a flax
  scutching machine: timber frame, a bladed beating
  drum under a partial housing, crank handle and feed
  plank.
- spinwheelitem (TOOL): "spinning wheel" -- the
  classic treadle wheel: large spoked drive wheel,
  bench beam, and the flyer/bobbin assembly on its
  uprights.
- steel_kiln_tool (TOOL): "tempering kiln" -- a small
  firebrick kiln box with banding straps, a chimney
  stub, and an arched fire door glowing orange.
- tablesaw_tool (TOOL): "table saw" -- a cabinet
  table saw: flat top with a rip fence, the toothed
  blade rising through the slot, cabinet base below.

Writes ONLY its own 10 folders + preview_item23.png.
PIL only; per-sprite random.Random(crc32(id)).
"""
import json
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
    "pseudo_magazine",
    "pseudo_punching_bag",
    "pseudo_training_dummy_heavy",
    "pseudo_training_dummy_light",
    "pseudo_treadmill_mechanical",
    "router_tool",
    "scutcheritem",
    "spinwheelitem",
    "steel_kiln_tool",
    "tablesaw_tool",
]

INK = (38, 34, 32, 255)


def canvas():
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    return img, ImageDraw.Draw(img)


def magazine(rng):
    img, d = canvas()
    red = (178, 44, 40, 255)
    red_d = (128, 30, 28, 255)
    red_l = (214, 84, 74, 255)
    # curved box magazine: polygon leaning with a bend
    body = [(12, 3), (19, 3), (20, 10), (21, 18), (22, 26),
            (15, 27), (14, 19), (13, 11)]
    d.polygon(body, fill=red, outline=INK)
    # left shaded wall
    d.polygon([(12, 3), (15, 3), (16, 11), (17, 19), (18, 26),
               (15, 27), (14, 19), (13, 11)], fill=red_d)
    # feed lips at the top
    d.rectangle([11, 1, 20, 3], fill=(96, 96, 102, 255), outline=INK)
    # horizontal ribs
    for y in (8, 13, 18, 23):
        d.line([(13, y), (20, y + 1)], fill=red_l)
    # baseplate
    d.rectangle([14, 26, 23, 29], fill=(70, 70, 76, 255), outline=INK)
    # charge bolt glyph on the flank
    bolt = (246, 200, 60, 255)
    d.polygon([(18, 12), (15, 18), (17, 18), (16, 23), (20, 16),
               (18, 16)], fill=bolt)
    return img


def punching_bag(rng):
    img, d = canvas()
    bag = (86, 86, 92, 255)
    bag_d = (58, 58, 64, 255)
    bag_l = (116, 116, 122, 255)
    chain = (150, 150, 156, 255)
    # hanger ring + chains
    d.ellipse([14, 0, 18, 4], outline=chain, width=1)
    d.line([(15, 3), (12, 8)], fill=chain)
    d.line([(17, 3), (20, 8)], fill=chain)
    d.line([(16, 3), (16, 8)], fill=chain)
    # bag body: rounded cylinder
    d.rounded_rectangle([10, 8, 22, 29], radius=4, fill=bag, outline=INK)
    # side shading
    d.rounded_rectangle([10, 8, 14, 29], radius=4, fill=bag_d)
    d.rectangle([11, 10, 13, 27], fill=bag_d)
    d.line([(18, 10), (18, 27)], fill=bag_l)
    # stitched seams
    for y in (13, 20):
        d.line([(11, y), (21, y)], fill=bag_d)
        for x in range(12, 21, 3):
            d.point((x, y - 1), fill=bag_l)
    # hanging straps over the crown
    d.line([(12, 8), (16, 11)], fill=bag_d)
    d.line([(20, 8), (16, 11)], fill=bag_d)
    # scuffs
    for _ in range(5):
        sx = rng.randrange(15, 21)
        sy = rng.randrange(15, 27)
        d.point((sx, sy), fill=bag_d)
    return img


def training_dummy(rng, armored):
    img, d = canvas()
    wood = (146, 108, 66, 255)
    wood_d = (110, 78, 44, 255)
    straw = (178, 148, 92, 255)
    steel = (168, 172, 178, 255)
    steel_d = (110, 114, 120, 255)
    # stand: post and cross feet
    d.rectangle([15, 20, 17, 29], fill=wood_d, outline=INK)
    d.rectangle([9, 29, 23, 30], fill=wood_d, outline=INK)
    d.rectangle([11, 27, 13, 29], fill=wood_d)
    d.rectangle([19, 27, 21, 29], fill=wood_d)
    # crossbar arms
    d.rectangle([5, 10, 27, 12], fill=wood, outline=INK)
    # torso: padded body
    d.rounded_rectangle([11, 8, 21, 22], radius=3, fill=straw, outline=INK)
    for _ in range(9):  # straw flecks
        sx = rng.randrange(12, 21)
        sy = rng.randrange(9, 22)
        d.point((sx, sy), fill=wood)
    # head
    d.ellipse([13, 1, 19, 8], fill=straw, outline=INK)
    if armored:
        # steel helmet dome
        d.pieslice([12, 0, 20, 8], start=180, end=360, fill=steel,
                   outline=INK)
        d.rectangle([12, 3, 20, 4], fill=steel_d)
        # riveted chest plate
        d.rectangle([12, 10, 20, 19], fill=steel, outline=INK)
        d.line([(12, 14), (20, 14)], fill=steel_d)
        for px in (13, 19):
            for py in (11, 18):
                d.point((px, py), fill=steel_d)
        # arm guards on the crossbar ends
        d.rectangle([5, 9, 9, 13], fill=steel, outline=INK)
        d.rectangle([23, 9, 27, 13], fill=steel, outline=INK)
    else:
        # simple rope bands on the torso
        d.line([(11, 13), (21, 13)], fill=wood_d)
        d.line([(11, 18), (21, 18)], fill=wood_d)
    return img


def treadmill(rng):
    img, d = canvas()
    frame = (84, 86, 92, 255)
    frame_d = (52, 54, 60, 255)
    belt = (40, 40, 44, 255)
    slat = (120, 122, 128, 255)
    # inclined belt deck (side view, front at right)
    d.polygon([(2, 20), (26, 24), (26, 28), (2, 24)], fill=belt,
              outline=INK)
    d.polygon([(2, 20), (26, 24), (26, 26), (2, 22)], fill=(58, 58, 64, 255))
    for i in range(6):  # belt slats
        x = 5 + i * 4
        y = 21 + (x - 2) // 8
        d.line([(x, y), (x, y + 3)], fill=slat)
    # mid support leg
    d.rectangle([12, 25, 14, 29], fill=frame_d, outline=INK)
    # rear roller
    d.ellipse([1, 19, 6, 25], fill=frame_d, outline=INK)
    d.point((3, 22), fill=slat)
    # front uprights and motor housing
    d.rectangle([24, 12, 26, 26], fill=frame, outline=INK)
    d.rectangle([21, 24, 29, 28], fill=frame_d, outline=INK)
    d.ellipse([23, 22, 28, 27], fill=frame, outline=INK)
    # handlebar
    d.line([(25, 12), (25, 8)], fill=frame)
    d.rectangle([18, 7, 29, 9], fill=frame, outline=INK)
    d.rectangle([18, 7, 20, 9], fill=frame_d)
    # small console nub on the bar
    d.rectangle([22, 5, 26, 7], fill=(64, 120, 140, 255), outline=INK)
    # feet
    d.rectangle([2, 27, 8, 29], fill=frame_d)
    return img


def router(rng):
    img, d = canvas()
    motor = (92, 112, 132, 255)
    motor_d = (60, 76, 96, 255)
    motor_l = (140, 160, 182, 255)
    grip = (52, 48, 46, 255)
    # knob handles
    d.ellipse([4, 11, 10, 17], fill=grip, outline=INK)
    d.ellipse([22, 11, 28, 17], fill=grip, outline=INK)
    d.rectangle([9, 13, 13, 15], fill=motor_d)
    d.rectangle([19, 13, 23, 15], fill=motor_d)
    # motor body
    d.rounded_rectangle([11, 4, 21, 19], radius=3, fill=motor, outline=INK)
    d.rectangle([13, 4, 15, 19], fill=motor_l)
    for y in (8, 11, 14):  # cooling slots
        d.line([(17, y), (20, y)], fill=motor_d)
    # top cap and cord stub
    d.rectangle([14, 2, 18, 4], fill=motor_d, outline=INK)
    d.line([(16, 2), (16, 0)], fill=grip)
    # base plate
    d.rectangle([8, 19, 24, 21], fill=(120, 122, 126, 255), outline=INK)
    # collet and bit
    d.rectangle([14, 21, 18, 23], fill=motor_d, outline=INK)
    d.polygon([(15, 23), (17, 23), (16, 28)], fill=(196, 198, 202, 255),
              outline=INK)
    return img


def scutcher(rng):
    img, d = canvas()
    wood = (138, 100, 60, 255)
    wood_d = (100, 70, 40, 255)
    steel = (150, 152, 156, 255)
    steel_d = (96, 98, 102, 255)
    # frame legs
    d.line([(7, 29), (10, 15)], fill=wood_d, width=2)
    d.line([(25, 29), (22, 15)], fill=wood_d, width=2)
    d.rectangle([5, 28, 27, 30], fill=wood_d, outline=INK)
    d.line([(8, 22), (24, 22)], fill=wood)
    # feed plank on the left
    d.polygon([(1, 15), (11, 13), (11, 16), (1, 18)], fill=wood,
              outline=INK)
    # drum housing: partial cover over the top
    d.pieslice([9, 5, 23, 19], start=180, end=360, fill=steel_d,
               outline=INK)
    d.rectangle([9, 12, 23, 13], fill=steel_d)
    # the bladed drum face
    d.ellipse([10, 7, 22, 19], fill=steel, outline=INK)
    for a in range(0, 360, 45):  # radial beater blades
        import math
        x = 16 + int(5.2 * math.cos(math.radians(a)))
        y = 13 + int(5.2 * math.sin(math.radians(a)))
        d.line([(16, 13), (x, y)], fill=steel_d)
    d.ellipse([14, 11, 18, 15], fill=steel_d, outline=INK)
    d.point((16, 13), fill=(220, 222, 226, 255))
    # crank handle on the right
    d.line([(22, 13), (27, 13)], fill=steel_d)
    d.line([(27, 13), (27, 17)], fill=steel_d)
    d.rectangle([26, 17, 29, 19], fill=wood, outline=INK)
    # beaten flax wisps under the drum
    for x in (11, 15, 20):
        d.line([(x, 24), (x + 1, 27)], fill=(196, 178, 120, 255))
    return img


def spinning_wheel(rng):
    img, d = canvas()
    wood = (140, 102, 62, 255)
    wood_d = (102, 72, 40, 255)
    wood_l = (178, 140, 92, 255)
    # bench beam and legs
    d.rectangle([4, 21, 29, 23], fill=wood, outline=INK)
    d.line([(7, 23), (5, 30)], fill=wood_d, width=2)
    d.line([(26, 23), (28, 30)], fill=wood_d, width=2)
    # treadle
    d.line([(12, 27), (22, 27)], fill=wood_d)
    d.line([(20, 23), (20, 27)], fill=wood_d)
    # drive wheel: rim, spokes, hub
    d.ellipse([2, 5, 18, 21], outline=wood_d, width=2)
    d.ellipse([3, 6, 17, 20], outline=wood_l, width=1)
    for a in range(0, 360, 45):
        import math
        x = 10 + int(6.6 * math.cos(math.radians(a)))
        y = 13 + int(6.6 * math.sin(math.radians(a)))
        d.line([(10, 13), (x, y)], fill=wood)
    d.ellipse([8, 11, 12, 15], fill=wood_d, outline=INK)
    # wheel support post
    d.rectangle([9, 19, 11, 22], fill=wood_d)
    # mother-of-all uprights on the right
    d.rectangle([23, 12, 25, 21], fill=wood_d, outline=INK)
    d.rectangle([28, 12, 30, 21], fill=wood_d, outline=INK)
    # flyer and bobbin between the uprights
    d.rectangle([24, 14, 29, 17], fill=(196, 178, 120, 255), outline=INK)
    d.line([(22, 13), (31, 13)], fill=steel_line(rng))
    d.polygon([(21, 11), (23, 13), (21, 15)], fill=wood_d)
    # wound yarn hint on the bobbin
    for x in (25, 27):
        d.line([(x, 14), (x, 17)], fill=(232, 226, 210, 255))
    return img


def steel_line(rng):
    return (120, 122, 126, 255)


def kiln(rng):
    img, d = canvas()
    brick = (156, 92, 68, 255)
    brick_d = (116, 64, 46, 255)
    strap = (70, 70, 76, 255)
    # chimney stub
    d.rectangle([19, 2, 24, 7], fill=brick_d, outline=INK)
    d.rectangle([18, 6, 25, 8], fill=strap, outline=INK)
    # kiln box
    d.rectangle([6, 8, 26, 30], fill=brick, outline=INK)
    # brick courses
    for y in (13, 19, 25):
        d.line([(6, y), (26, y)], fill=brick_d)
    for x in (11, 17, 23):
        d.line([(x, 8), (x, 13)], fill=brick_d)
        d.line([(x + 2, 19), (x + 2, 25)], fill=brick_d)
    # banding straps
    d.rectangle([6, 10, 26, 11], fill=strap)
    d.rectangle([6, 27, 26, 28], fill=strap)
    # arched fire door with glow
    d.pieslice([11, 14, 21, 24], start=180, end=360, fill=(60, 40, 34, 255))
    d.rectangle([11, 19, 21, 29], fill=(60, 40, 34, 255), outline=INK)
    d.pieslice([12, 15, 20, 23], start=180, end=360, fill=(238, 138, 44, 255))
    d.rectangle([12, 19, 20, 28], fill=(238, 138, 44, 255))
    d.rectangle([14, 21, 18, 28], fill=(250, 196, 84, 255))
    d.rectangle([15, 24, 17, 28], fill=(252, 230, 160, 255))
    # door frame lip
    d.rectangle([10, 28, 22, 30], fill=strap, outline=INK)
    return img


def tablesaw(rng):
    img, d = canvas()
    top = (176, 178, 182, 255)
    top_d = (128, 130, 134, 255)
    cab = (104, 108, 114, 255)
    cab_d = (70, 74, 80, 255)
    blade = (206, 208, 212, 255)
    # cabinet base
    d.rectangle([9, 17, 24, 30], fill=cab, outline=INK)
    d.rectangle([12, 20, 21, 27], fill=cab_d, outline=INK)
    d.rectangle([13, 21, 20, 26], fill=cab, outline=INK)
    d.point((19, 23), fill=(220, 222, 226, 255))
    # tabletop slab
    d.polygon([(3, 12), (29, 12), (26, 17), (6, 17)], fill=top, outline=INK)
    d.line([(6, 15), (26, 15)], fill=top_d)
    # blade slot and the blade rising through it
    d.ellipse([12, 3, 21, 12], fill=blade, outline=INK)
    for a in range(0, 360, 30):  # teeth
        import math
        x = 16 + int(4.8 * math.cos(math.radians(a)))
        y = 7 + int(4.8 * math.sin(math.radians(a)))
        d.point((x, y), fill=(140, 142, 146, 255))
    d.ellipse([15, 6, 18, 9], fill=(140, 142, 146, 255), outline=INK)
    d.rectangle([13, 11, 20, 13], fill=blade)
    d.line([(11, 12), (22, 12)], fill=cab_d)
    # rip fence to the right of the blade
    d.polygon([(23, 10), (27, 10), (25, 15), (21, 15)], fill=(150, 128, 84, 255),
              outline=INK)
    # miter gauge track lines
    d.line([(5, 13), (10, 13)], fill=top_d)
    return img


def render(item_id, rng):
    if item_id == "pseudo_magazine":
        return magazine(rng)
    if item_id == "pseudo_punching_bag":
        return punching_bag(rng)
    if item_id == "pseudo_training_dummy_heavy":
        return training_dummy(rng, True)
    if item_id == "pseudo_training_dummy_light":
        return training_dummy(rng, False)
    if item_id == "pseudo_treadmill_mechanical":
        return treadmill(rng)
    if item_id == "router_tool":
        return router(rng)
    if item_id == "scutcheritem":
        return scutcher(rng)
    if item_id == "spinwheelitem":
        return spinning_wheel(rng)
    if item_id == "steel_kiln_tool":
        return kiln(rng)
    if item_id == "tablesaw_tool":
        return tablesaw(rng)
    raise ValueError(item_id)


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
    cols, cell = 5, 66
    rows = (len(IDS) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * cell, rows * cell), (24, 24, 28, 255))
    for i, item_id in enumerate(IDS):
        big = imgs[item_id].resize((64, 64), Image.NEAREST)
        sheet.alpha_composite(big, ((i % cols) * cell + 1,
                                    (i // cols) * cell + 1))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item23.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item23.png {sheet.size}")


if __name__ == "__main__":
    main()
