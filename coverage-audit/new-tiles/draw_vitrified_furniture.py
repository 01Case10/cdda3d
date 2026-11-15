#!/usr/bin/env python3
"""
Draw the missing vitrified-furniture sprites from their game descriptions.

Sources (fork-src/cdda/data/json/furniture_and_terrain/
furniture_vitrified.json, CDDA 6a3e59e0) -- the 32 ids below are the
entries in that file listed as missing in the shipped universe
(coverage-audit/missing.json, category "furniture"); the file's other
entries resolve via looks_like and stay out:

  f_vitrified_bed          "black glass sculpture of a bed.  The rumpled
                            covers are jagged, and splinter at the touch."
  f_vitrified_bathtub      "tub made from black glass, with a frozen
                            column of onyx pouring from the tap."
  f_vitrified_dresser      "black glass cabinet with its doors slightly
                            ajar.  Inside is a row of hangers bearing
                            neatly folded and entirely rigid clothing."
  f_vitrified_fridge       "black glass refrigerator, its door hanging
                            open.  Inside is a seamless surface with
                            protrusions resembling fresh fruit and
                            plastic bottles."
  f_vitrified_armchair     "once very plush and comfortable.  Now the
                            overstuffed cushions are hard, cold, and
                            slippery."
  f_vitrified_chair        "simple glass chair.  The seat is very thin,
                            and crunches ominously under any weight."
  f_vitrified_desk         "glass desk.  Shapes atop it suggest pens,
                            paper, and miscellaneous knickknacks too
                            jagged to identify."
  f_vitrified_locker       "tall glass cabinet.  The door is seamless
                            with the frame."
  f_vitrified_oven         "glass sculpture of a gas oven.  Bristling
                            from the burner is a crown of jagged shards
                            in the rough shape of an uncontrolled fire."
  f_vitrified_mailbox      "glass sculpture of a mailbox.  The door is
                            open; inside is a pile of envelopes, all
                            razor-edged."  color light_gray
  f_vitrified_rack         "glass sculpture of a shelving unit.  The
                            angled shelves are covered in jagged
                            miscellany, fused in place."  light_gray
  f_vitrified_sink         "glass basin and tap.  The inside stops
                            halfway down in a sheet of concentric
                            ripples, with the tiny spire of a splashing
                            droplet at the center."
  f_vitrified_toilet       "toilet made of black glass.  The lid,
                            perhaps mercifully, is fused shut."
  f_vitrified_cupboard     "black glass cupboard.  The doors are fused
                            shut, but it presumably contains glasses."
  f_vitrified_planter      "planter of dark glass.  Sharp, twisting
                            glass sprouts emerge from the black soil."
  f_vitrified_bench        "long, low slab of black glass.  The fissures
                            on the surface might be natural cracks, or
                            carved names."
  f_vitrified_washer       "glass washing machine.  The inside is
                            completely filled with black sand, some of
                            which has spilled onto the floor."
  f_vitrified_dryer        "mechanical dryer of black glass.  The door
                            hangs open; inside, a jumble of what might
                            once have been clothes, smashed to pieces."
  f_vitrified_bookcase     "heavy bookshelf, laden with books fused
                            into the shelves and each other.  Titles on
                            the spines, in an unfamiliar alphabet."
  f_vitrified_bookcase_talker  same bookcase (looks_like the plain
                            bookcase): "This bookcase is missing a book."
  f_vitrified_counter      "thick cube of black glass.  On the surface,
                            fused in place, are statues of utensils and
                            miscellany."
  f_vitrified_table        "slab of smooth black glass on four stout
                            legs.  Shallowly rounded masses, like liquid
                            spilled on the surface."
  f_vitrified_birdbath     "glass pedestal and bowl.  Inside, frozen
                            mid-wash in the black sand, is the delicate
                            figure of a bird."  color light_gray
  f_vitrified_shower       "from the head descend a dozen streams of
                            thin black glass, with frozen droplets
                            strung from them like pearls."
  f_vitrified_water_heater "cylinder of thick black glass.  The top has
                            burst apart; rivulets of black sand run
                            down the side."
  f_vitrified_fireplace    "fireplace of pitted glass bricks.  The fire
                            inside has been frozen in glass at the moment
                            it began to leap beyond the grate."
  f_vitrified_firering     "circle of smooth glass pebbles.  Inside are
                            frozen tendrils of fire, slightly translucent."
  f_vitrified_floor_lamp   "black glass lantern: a tall glass pole with
                            a translucent shade atop it; in the corner of
                            your eye, a soft glow."  color light_gray
  f_vitrified_box          "box of thin black glass.  Its edges are
                            glisteningly sharp."
  f_vitrified_sofa         "long bench, padded with cold glass cushions."
  f_vitrified_dishwasher   "glass dishwasher, its door hanging open.
                            Inside, glass plates and utensils are half
                            buried in black sand."
  f_vitrified_doll         "incomplete plush doll: a soft plush torso,
                            empty of features, lying on this sofa." --
                            the one non-glass object in the set: a matte
                            cloth torso on a glass sofa.

Design: UltiCa decomposed layout, all sprites 32x32 on the normal
sheet, drawn as objects on a transparent background (bg "" -- furniture
draws over the terrain beneath it). The glass language matches
draw_vitrified_terrain.py exactly: near-black blue-teal glass rendered
OPAQUE (dark bodies, pale teal/gray specular highlights), the
light_gray ids shifted one tone paler. Semi-transparent pixels are
never drawn into a glass body; the few genuinely translucent effects
(the firering's frozen fire, the lamp's glow and shade) are painted on
a separate overlay and composited with Image.alpha_composite.
Deterministic: every random choice comes from a per-sprite seeded
Random.
Output: pngs_normal_32x32/furniture/<id>/ (png + tile-entry json +
credits) and preview_vitrified_furniture.png.
"""
from PIL import Image, ImageDraw
import json, math, os, random

BASE = os.path.dirname(os.path.abspath(__file__))
S = 32

def new_img():
    return Image.new("RGBA", (S, S), (0, 0, 0, 0))

# black-glass palette -- identical to draw_vitrified_terrain.py
GLASS_DEEP = (14, 19, 22, 255)
GLASS_DARK = (24, 32, 36, 255)
GLASS_MID = (44, 60, 64, 255)
GLASS_PALE = (96, 128, 128, 255)
GLASS_HI = (168, 204, 198, 255)
GLASS_SHEEN = (210, 238, 230, 255)
EDGE = (6, 8, 10, 255)
SEAM = (52, 68, 68, 255)
SAND = (8, 10, 12, 255)  # black sand / onyx

DARK_PAL = {"body": GLASS_DARK, "body2": (30, 40, 44, 255),
            "facet": GLASS_MID, "pale": GLASS_PALE, "hi": GLASS_HI,
            "deep": GLASS_DEEP}
LIGHT_PAL = {"body": GLASS_MID, "body2": (56, 74, 76, 255),
             "facet": GLASS_PALE, "pale": (140, 172, 168, 255),
             "hi": GLASS_SHEEN, "deep": GLASS_DARK}

def rrect(d, xy, radius=2, fill=None):
    """Float-safe rounded rectangle (this Pillow build's
    ImageDraw.rounded_rectangle rejects some float boxes)."""
    x0, y0, x1, y1 = (float(v) for v in xy)
    r = max(0.1, min(radius, (x1 - x0) / 2, (y1 - y0) / 2))
    d.rectangle([x0 + r, y0, x1 - r, y1], fill=fill)
    d.rectangle([x0, y0 + r, x1, y1 - r], fill=fill)
    for (cx, cy) in [(x0 + r, y0 + r), (x1 - r, y0 + r),
                     (x0 + r, y1 - r), (x1 - r, y1 - r)]:
        d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=fill)

def glints(d, rng, n, box=(0, 0, S - 1, S - 1), col=GLASS_HI):
    """Sparse pale specular flecks -- the glassy catch-light."""
    for _ in range(n):
        x = rng.uniform(box[0], box[2]); y = rng.uniform(box[1], box[3])
        d.point((x, y), fill=col)

def sheen(img, rng, n=1):
    """Broad diagonal specular streaks, composited (never drawn into
    the body with a bare semi-transparent fill)."""
    ov = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    od = ImageDraw.Draw(ov)
    for _ in range(n):
        x = rng.uniform(2, 18); y = rng.uniform(10, 26)
        od.polygon([(x, y), (x + 8, y - 7), (x + 11, y - 7), (x + 3, y)],
                   fill=(190, 224, 214, 40))
    img.alpha_composite(ov)

def panel(d, x0, y0, x1, y1, pal, body=None):
    """A front-facing glass panel: body fill, pale top light, deep
    right/bottom shade."""
    d.rectangle([x0, y0, x1, y1], fill=body or pal["body"])
    d.line([x0, y0, x1, y0], fill=pal["pale"], width=1)
    d.line([x0, y0, x0, y1], fill=pal["facet"], width=1)
    d.line([x1, y0, x1, y1], fill=pal["deep"], width=1)
    d.line([x0, y1, x1, y1], fill=pal["deep"], width=1)

def cracks(d, rng, box, n, pal):
    """Thin jagged fissures: a deep line with a pale twin offset."""
    for _ in range(n):
        x = rng.uniform(box[0], box[2] - 4); y = rng.uniform(box[1], box[3])
        pts = [(x, y)]
        for _ in range(rng.randint(2, 3)):
            x += rng.uniform(1.5, 3.5); y += rng.uniform(-1.5, 1.5)
            pts.append((x, y))
        d.line(pts, fill=pal["deep"], width=1)
        d.line([(px, py - 1) for px, py in pts], fill=pal["facet"], width=1)

# ---------------------------------------------------------------- bed
def draw_bed():
    rng = random.Random(3401); pal = DARK_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    # headboard (left), with slats
    panel(d, 2.5, 5, 8.5, 25, pal)
    for y in (10.5, 15.5, 20.5):
        d.line([3.5, y, 7.5, y], fill=pal["deep"], width=1)
    d.line([3.5, 6, 3.5, 24], fill=pal["pale"], width=1)
    # mattress top plane, then the front face of the frame
    d.polygon([(8, 13), (29, 13), (31, 17), (10, 17)], fill=pal["facet"])
    d.polygon([(10, 17), (31, 17), (31, 24), (10, 24)], fill=pal["body"])
    d.line([10, 24, 31, 24], fill=pal["deep"], width=1)
    # pillow lump at the head: bright enough to read as a pillow
    d.ellipse([8.8, 10.2, 16.4, 14.8], fill=(120, 152, 148, 255))
    d.ellipse([9.8, 10.9, 15.4, 13.9], fill=pal["pale"])
    d.arc([9.8, 10.9, 15.4, 13.9], start=180, end=330, fill=pal["hi"], width=1)
    # the covers' near edge: a pale fold line along the front face
    d.line([10.5, 19.5, 30.5, 19.5], fill=pal["facet"], width=1)
    d.line([10.5, 20.3, 30.5, 20.3], fill=pal["deep"], width=1)
    # rumpled covers: jagged frozen folds over the mattress
    ridge = [(16, 13.5), (19, 11), (21.5, 14), (24, 11.8), (26.5, 14.2),
             (29.5, 12.4), (31, 15), (31, 19), (24, 18), (19, 19.5),
             (14, 18.4), (11, 19), (11, 15)]
    d.polygon(ridge, fill=pal["body2"])
    for (x, y) in [(19, 11), (24, 11.8), (29.5, 12.4)]:
        d.line([x, y, x + 0.8, y + 2.6], fill=pal["hi"], width=1)
    # splinters at the touch: small shards sticking off the cover edge
    for (x, y, a) in [(13, 18.6, -0.7), (22, 18.6, 0.4), (28, 17.8, -0.4)]:
        d.polygon([(x, y), (x + 2.2, y), (x + 1 + a, y + 2.6)],
                  fill=pal["facet"], outline=pal["hi"])
    # legs
    d.rectangle([11, 24, 13, 29], fill=pal["deep"])
    d.rectangle([27, 24, 29, 29], fill=pal["deep"])
    sheen(img, rng, 1); glints(d, rng, 6, box=(10, 12, 30, 23))
    return img

# ---------------------------------------------------------------- bathtub
def draw_bathtub():
    rng = random.Random(3402); pal = DARK_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    # tap at the right end, drawn first (behind the rim)
    d.rectangle([26.5, 3, 28.5, 10], fill=pal["body"])
    d.line([27.5, 3, 27.5, 10], fill=pal["pale"], width=1)
    d.rectangle([23, 3, 28.5, 5], fill=pal["body"])  # spout arm
    d.point((24, 4), fill=pal["hi"])
    # tub body tapering down from the rim
    d.polygon([(3, 13), (29, 13), (26, 25), (6, 25)], fill=pal["body"])
    d.line([5, 18, 27, 18], fill=pal["facet"], width=1)
    d.line([4.2, 13, 6.5, 24], fill=pal["facet"], width=1)
    # feet
    d.polygon([(7, 25), (10.5, 25), (10, 29), (7.5, 29)], fill=pal["deep"])
    d.polygon([(21.5, 25), (25, 25), (24.5, 29), (22, 29)], fill=pal["deep"])
    # rim and the glass inside
    d.ellipse([2, 9.5, 30, 17.5], fill=pal["facet"], outline=pal["pale"])
    d.ellipse([5, 11.5, 27, 16], fill=pal["deep"])
    d.arc([5, 11.5, 27, 16], start=200, end=330, fill=pal["pale"], width=1)
    # frozen column of onyx pouring from the tap into the tub
    d.polygon([(23.4, 5), (25.6, 5), (26.1, 13.5), (22.9, 13.5)],
              fill=(8, 10, 13, 255))
    d.line([24.2, 5.5, 24.6, 13], fill=(120, 150, 146, 255), width=1)
    d.ellipse([21.5, 12.6, 27.5, 14.6], fill=(10, 13, 16, 255))
    sheen(img, rng, 1); glints(d, rng, 6, box=(4, 14, 28, 24))
    return img

# ---------------------------------------------------------------- dresser
def draw_dresser():
    rng = random.Random(3403); pal = DARK_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    panel(d, 5, 4, 27, 29, pal)
    # dark interior showing between the ajar doors
    d.rectangle([9, 7, 23, 26], fill=pal["deep"])
    # hanger rod + a row of hangers with rigid folded clothing
    d.line([10, 10, 22, 10], fill=pal["pale"], width=1)
    for hx in (11.5, 15, 18.5, 21.5):
        d.polygon([(hx, 10), (hx - 2, 13), (hx + 2, 13)], outline=pal["hi"])
        d.rectangle([hx - 1.8, 13, hx + 1.8, 19], fill=pal["facet"])
        d.line([hx - 1.8, 15.5, hx + 1.8, 15.5], fill=pal["pale"], width=1)
        d.line([hx - 1.8, 17.5, hx + 1.8, 17.5], fill=pal["deep"], width=1)
    # the two doors, swung slightly outward toward the viewer
    d.polygon([(5, 5), (9.5, 7), (9.5, 26), (5, 28)], fill=pal["body2"])
    d.line([9.5, 7, 9.5, 26], fill=pal["hi"], width=1)
    d.polygon([(27, 5), (22.5, 7), (22.5, 26), (27, 28)], fill=pal["body2"])
    d.line([22.5, 7, 22.5, 26], fill=pal["hi"], width=1)
    glints(d, rng, 5, box=(6, 5, 26, 28))
    return img

# ---------------------------------------------------------------- fridge
def draw_fridge():
    rng = random.Random(3404); pal = DARK_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    panel(d, 8, 3, 24, 29, pal)
    # interior back: a seamless surface with protrusions
    d.rectangle([10, 5, 23, 27], fill=(34, 46, 48, 255))
    d.line([10, 15, 23, 15], fill=pal["pale"], width=1)  # shelf edges
    d.line([10, 22, 23, 22], fill=pal["pale"], width=1)
    # fruit: fused spheres on the middle shelf
    for (x, y, r) in [(12.5, 13, 1.9), (16, 12.4, 2.2), (20, 13.2, 1.7)]:
        d.ellipse([x - r, y - r, x + r, y + r], fill=pal["facet"])
        d.point((x - 0.6, y - 0.7), fill=pal["hi"])
    # bottles on the lower shelf
    for bx in (12.5, 17.5):
        d.rectangle([bx, 17.5, bx + 2.6, 21.8], fill=pal["pale"])
        d.rectangle([bx + 0.7, 15.8, bx + 1.9, 17.5], fill=pal["pale"])
        d.line([bx + 0.5, 18.5, bx + 0.5, 21], fill=pal["hi"], width=1)
    d.ellipse([20.5, 19.5, 22.8, 21.8], fill=pal["facet"])  # one more fruit
    # the door, hanging open to the right
    d.polygon([(24, 5), (30.5, 8), (30.5, 26), (24, 29)], fill=pal["body2"])
    d.polygon([(25.4, 8), (29.2, 10), (29.2, 23.5), (25.4, 26)],
              fill=pal["facet"])
    d.line([24.6, 6, 24.6, 28], fill=pal["hi"], width=1)  # door edge light
    # freezer seam on the part of the body still visible
    d.line([8, 9, 10, 9], fill=pal["deep"], width=1)
    glints(d, rng, 5, box=(9, 4, 23, 28))
    return img

# ---------------------------------------------------------------- armchair
def draw_armchair():
    rng = random.Random(3405); pal = DARK_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    # plush back, then the overstuffed inner cushion
    rrect(d, [7, 4, 25, 21], radius=4, fill=pal["body"])
    rrect(d, [10.5, 7, 21.5, 17], radius=3, fill=pal["facet"])
    d.arc([10.5, 7, 21.5, 17], start=180, end=330, fill=pal["pale"], width=1)
    # rolled arms
    for (x0, x1) in [(3.5, 10), (22, 28.5)]:
        rrect(d, [x0, 12, x1, 26], radius=3, fill=pal["body2"])
        d.ellipse([x0 + 0.5, 11, x1 - 0.5, 16], fill=pal["facet"])
        d.arc([x0 + 0.5, 11, x1 - 0.5, 16], start=180, end=330,
              fill=pal["pale"], width=1)
    # seat cushion and base
    rrect(d, [9.5, 17.5, 22.5, 23], radius=2, fill=pal["facet"])
    d.rectangle([6.5, 23, 25.5, 27.5], fill=pal["body"])
    d.rectangle([8, 27.5, 10.5, 30], fill=pal["deep"])
    d.rectangle([21.5, 27.5, 24, 30], fill=pal["deep"])
    # cushion seams: hard, slippery segments
    d.line([16, 8, 16, 16.4], fill=pal["deep"], width=1)
    d.line([16, 18.2, 16, 22.4], fill=pal["deep"], width=1)
    sheen(img, rng, 1); glints(d, rng, 6, box=(5, 6, 27, 26))
    return img

# ---------------------------------------------------------------- chair
def draw_chair():
    rng = random.Random(3406); pal = DARK_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    # back posts and two slats
    d.rectangle([10, 3, 12, 19], fill=pal["body"])
    d.rectangle([20, 3, 22, 19], fill=pal["body"])
    d.rectangle([10, 3.5, 22, 6.5], fill=pal["facet"])
    d.rectangle([10, 9.5, 22, 12], fill=pal["facet"])
    d.line([10, 3.5, 22, 3.5], fill=pal["pale"], width=1)
    # the very thin seat
    d.polygon([(7.5, 18), (24.5, 18), (25.5, 19.6), (6.5, 19.6)],
              fill=pal["pale"])
    d.line([6.5, 19.6, 25.5, 19.6], fill=pal["deep"], width=1)
    # thin legs, slightly splayed
    d.polygon([(8.5, 19.6), (10.5, 19.6), (10, 29.5), (8, 29.5)],
              fill=pal["body"])
    d.polygon([(21.5, 19.6), (23.5, 19.6), (24, 29.5), (22, 29.5)],
              fill=pal["body"])
    d.line([9.2, 20, 8.8, 29], fill=pal["facet"], width=1)
    # an ominous hairline crack across the seat
    d.line([12, 18.4, 15, 19, 18, 18.5], fill=pal["deep"], width=1)
    glints(d, rng, 4, box=(8, 4, 24, 19))
    return img

# ---------------------------------------------------------------- desk
def draw_desk():
    rng = random.Random(3407); pal = DARK_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    # pedestals first, then the top slab over them
    panel(d, 4, 16, 11, 29, pal)
    panel(d, 21, 16, 28, 29, pal)
    d.line([5.5, 20.5, 9.5, 20.5], fill=pal["deep"], width=1)  # drawer seams
    d.line([5.5, 24.5, 9.5, 24.5], fill=pal["deep"], width=1)
    d.point((7.5, 18.5), fill=pal["hi"]); d.point((7.5, 22.5), fill=pal["hi"])
    d.rectangle([12, 17, 20, 29], fill=pal["deep"])  # knee hole shadow
    d.polygon([(2.5, 12.5), (29.5, 12.5), (30.5, 15.5), (1.5, 15.5)],
              fill=pal["facet"])
    d.line([2.5, 12.5, 29.5, 12.5], fill=pal["pale"], width=1)
    # atop it: a sheet of paper (pale, jagged), a pen, knickknack shards
    d.polygon([(11, 7.5), (19, 8.5), (18.2, 12.3), (11.8, 11.6)],
              fill=(148, 178, 170, 255), outline=pal["hi"])
    d.line([12.5, 9.5, 17.5, 10.2], fill=pal["deep"], width=1)
    d.line([21, 11.8, 25.5, 9.6], fill=pal["hi"], width=1)  # pen sliver
    for (x, y, s) in [(23.5, 12, 2.2), (26.5, 10.8, 1.7), (6.5, 11, 1.9)]:
        d.polygon([(x - s, y + s * 0.6), (x + s, y + s * 0.6), (x, y - s)],
                  fill=pal["body2"], outline=pal["pale"])
    glints(d, rng, 5, box=(3, 13, 29, 28))
    return img

# ---------------------------------------------------------------- locker
def draw_locker():
    rng = random.Random(3408); pal = DARK_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    panel(d, 9, 2, 23, 30, pal)
    # the seamless door: a seam you have to look for
    d.rectangle([11, 4.5, 21, 28], outline=(38, 50, 52, 255), width=1)
    # vent slits near the top
    for y in (6.5, 8.5):
        d.line([13, y, 19, y], fill=pal["deep"], width=1)
    # a handle nub, fused like everything else
    d.ellipse([19, 15.5, 20.8, 17.5], fill=pal["facet"], outline=pal["pale"])
    sheen(img, rng, 1); glints(d, rng, 5, box=(10, 3, 22, 29))
    return img

# ---------------------------------------------------------------- oven
def draw_oven():
    rng = random.Random(3409); pal = DARK_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    panel(d, 5, 9, 27, 29, pal)
    # door with a dark window band and a handle bar
    d.rectangle([7.5, 15, 24.5, 26], fill=pal["deep"])
    d.rectangle([9, 17.5, 23, 21.5], fill=(16, 22, 25, 255),
                outline=pal["facet"])
    d.rectangle([7, 12, 25, 13.4], fill=pal["pale"])
    d.point((9, 10.8), fill=pal["hi"]); d.point((22, 10.8), fill=pal["hi"])
    # the crown of jagged shards, in the rough shape of uncontrolled fire
    crown = [(6, 9), (7.5, 3.5), (9.5, 8), (11, 1.5), (13, 7.5), (15, 3),
             (16.5, 8.5), (18.5, 2), (20.5, 7.5), (22, 4), (24, 8.5),
             (25.5, 5.5), (26.5, 9)]
    d.polygon(crown, fill=pal["facet"])
    inner = [(10, 8.6), (12, 5), (14.5, 8.4), (17, 4.6), (19.5, 8.6),
             (21.5, 6), (23, 8.8)]
    d.polygon(inner, fill=pal["pale"])
    for (x, y) in [(11, 2.6), (18.5, 3.2), (22, 5)]:
        d.point((x, y), fill=pal["hi"])
    d.line([6, 9, 26.5, 9], fill=pal["hi"], width=1)
    glints(d, rng, 5, box=(6, 10, 26, 28))
    return img

# ---------------------------------------------------------------- mailbox
def draw_mailbox():
    rng = random.Random(3410); pal = LIGHT_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    # post
    d.rectangle([14.5, 19, 17.5, 30], fill=DARK_PAL["body"])
    d.line([15.2, 19, 15.2, 30], fill=DARK_PAL["facet"], width=1)
    # tunnel body: rounded top over straight sides
    d.rectangle([6, 11, 26, 20], fill=pal["body"])
    d.pieslice([6, 3, 26, 19], start=180, end=360, fill=pal["body"])
    d.arc([6, 3, 26, 19], start=180, end=360, fill=pal["hi"], width=1)
    d.line([6, 12, 6, 20], fill=pal["facet"], width=1)
    d.line([26, 12, 26, 20], fill=pal["deep"], width=1)
    d.rectangle([6, 19, 26, 20.5], fill=pal["deep"])
    # the open front: dark mouth at the right end, envelopes spilling out
    d.ellipse([20.5, 9, 27.5, 17], fill=pal["deep"])
    for (x, y, w, a) in [(21.5, 8.5, 4.5, -0.5), (23, 10.5, 4, 0.4),
                         (22, 12.5, 4.6, -0.2)]:
        pts = [(x, y), (x + w, y + a), (x + w - 0.4, y + 3), (x + 0.3, y + 3.2)]
        d.polygon(pts, fill=(196, 222, 212, 255), outline=pal["hi"])
    # the door, swung down below the mouth
    d.polygon([(21, 16.5), (27, 16.5), (28.5, 21), (22, 21.5)],
              fill=pal["facet"], outline=pal["hi"])
    glints(d, rng, 6, box=(7, 5, 25, 19), col=pal["hi"])
    return img

# ---------------------------------------------------------------- rack
def draw_rack():
    rng = random.Random(3411); pal = LIGHT_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    # uprights
    d.rectangle([4.5, 3.5, 6.5, 29], fill=pal["body"])
    d.rectangle([25.5, 3.5, 27.5, 29], fill=pal["body"])
    d.line([5.2, 3.5, 5.2, 29], fill=pal["hi"], width=1)
    tones = [pal["facet"], pal["body2"], (120, 152, 148, 255), pal["deep"]]
    # three angled shelves, each covered in fused jagged miscellany
    for (y0, y1) in [(9, 12.5), (17, 20.5), (25, 28.5)]:
        for _ in range(5):  # miscellany sits behind the shelf lip
            x = rng.uniform(8, 23); h = rng.uniform(2.2, 4.2)
            w = rng.uniform(1.8, 3.2)
            col = tones[rng.randrange(len(tones))]
            d.polygon([(x - w / 2, y0 + 1.5), (x, y0 + 1.5 - h),
                       (x + w / 2, y0 + 1.5), (x + w * 0.2, y0 + 2.2)],
                      fill=col, outline=pal["hi"])
        d.polygon([(5, y0 + 1), (27, y0 - 1.2), (27, y1 - 1.2), (5, y1 + 1)],
                  fill=pal["facet"])
        d.line([5, y0 + 1, 27, y0 - 1.2], fill=pal["hi"], width=1)
    glints(d, rng, 6, box=(6, 5, 26, 28), col=pal["hi"])
    return img

# ---------------------------------------------------------------- sink
def draw_sink():
    rng = random.Random(3412); pal = DARK_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    # tap: riser + curved spout over the basin
    d.rectangle([14.8, 3.5, 17.2, 10], fill=pal["body"])
    d.arc([14.5, 2.5, 23, 11], start=270, end=380, fill=pal["pale"], width=2)
    d.ellipse([20.6, 8.6, 22.6, 10.4], fill=pal["facet"])
    d.point((15.6, 4.2), fill=pal["hi"])
    # pedestal
    d.polygon([(13, 18), (19, 18), (20.5, 29), (11.5, 29)], fill=pal["body"])
    d.line([14.5, 19, 15.5, 28], fill=pal["facet"], width=1)
    # basin rim and interior
    d.ellipse([3.5, 10.5, 28.5, 19.5], fill=pal["facet"], outline=pal["pale"])
    d.ellipse([6.5, 12.3, 25.5, 18], fill=pal["deep"])
    # the frozen sheet of concentric ripples, stopping halfway down
    for r in (7.6, 5.4, 3.4):
        d.arc([16 - r, 15.1 - r * 0.42, 16 + r, 15.1 + r * 0.42],
              start=15, end=345, fill=pal["pale"], width=1)
    # the tiny spire of the splashing droplet at the center
    d.polygon([(15.2, 15.4), (16.8, 15.4), (16, 11.8)], fill=pal["hi"])
    d.ellipse([15.5, 10.6, 16.5, 11.8], fill=pal["hi"])
    glints(d, rng, 5, box=(5, 11, 27, 19))
    return img

# ---------------------------------------------------------------- toilet
def draw_toilet():
    rng = random.Random(3413); pal = DARK_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    # tank
    panel(d, 9, 3, 23, 10.5, pal)
    d.line([9, 5.5, 23, 5.5], fill=pal["deep"], width=1)  # tank lid seam
    d.rectangle([14.8, 10.5, 17.2, 14], fill=pal["body"])  # tank-to-bowl neck
    # bowl with the lid fused shut: smooth oval, one seam line
    d.ellipse([6.5, 14, 25.5, 23.5], fill=pal["facet"], outline=pal["pale"])
    d.ellipse([8.6, 15.6, 23.4, 21.9], outline=pal["pale"], width=1)
    d.arc([8.6, 15.6, 23.4, 21.9], start=200, end=320, fill=pal["hi"], width=1)
    # pedestal base
    d.polygon([(12.5, 23), (19.5, 23), (20.5, 29), (11.5, 29)],
              fill=pal["body"])
    d.line([14, 24, 14.8, 28.4], fill=pal["facet"], width=1)
    glints(d, rng, 5, box=(8, 4, 24, 22))
    return img

# ---------------------------------------------------------------- cupboard
def draw_cupboard():
    rng = random.Random(3414); pal = DARK_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    panel(d, 5, 5, 27, 25, pal)
    d.rectangle([4, 3, 28, 5.4], fill=pal["facet"])  # cornice
    d.line([4, 3, 28, 3], fill=pal["pale"], width=1)
    # two doors, fused shut: inset panels, one center seam, two knobs
    d.rectangle([7.5, 8, 14.8, 22], outline=pal["facet"], width=1)
    d.rectangle([17.2, 8, 24.5, 22], outline=pal["facet"], width=1)
    d.line([16, 6, 16, 24], fill=pal["deep"], width=1)
    d.point((14.6, 15), fill=pal["hi"]); d.point((17.4, 15), fill=pal["hi"])
    # the glasses it presumably contains: faint shapes behind the glass
    for x in (10, 20.5):
        d.rectangle([x, 17.5, x + 2.2, 21], outline=(60, 78, 76, 255), width=1)
    d.rectangle([5, 25, 27, 27], fill=pal["deep"])  # under-shadow
    glints(d, rng, 5, box=(6, 6, 26, 24))
    return img

# ---------------------------------------------------------------- planter
def draw_planter():
    rng = random.Random(3415); pal = DARK_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    # sharp, twisting glass sprouts from the black soil
    for (bx, h) in [(9, 12), (13.5, 15), (18, 11), (22.5, 14), (16, 8.5)]:
        pts = []
        for i in range(9):
            t = i / 8
            pts.append((bx + 1.8 * math.sin(t * 4.2 + bx), 18 - h * t))
        d.line(pts, fill=pal["facet"], width=2)
        d.line([(x - 0.7, y) for x, y in pts], fill=pal["pale"], width=1)
        tx, ty = pts[-1]
        d.polygon([(tx - 1.4, ty + 1.6), (tx + 1.4, ty + 1.6), (tx, ty - 2)],
                  fill=pal["pale"], outline=pal["hi"])  # sharp tip leaf
    # trough
    d.polygon([(4, 17.5), (28, 17.5), (25, 27), (7, 27)], fill=pal["body"])
    d.rectangle([3, 15.5, 29, 17.8], fill=pal["facet"])
    d.line([3, 15.5, 29, 15.5], fill=pal["pale"], width=1)
    d.rectangle([6.5, 16.2, 25.5, 17.4], fill=SAND)  # black soil surface
    d.line([7, 26, 25, 26], fill=pal["deep"], width=1)
    glints(d, rng, 5, box=(5, 18, 27, 26))
    return img

# ---------------------------------------------------------------- bench
def draw_bench():
    rng = random.Random(3416); pal = DARK_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    # long, low slab: top plane and front face
    d.polygon([(2, 14.5), (30, 14.5), (31, 17.5), (1, 17.5)],
              fill=pal["facet"])
    d.line([2, 14.5, 30, 14.5], fill=pal["pale"], width=1)
    d.rectangle([1, 17.5, 31, 21], fill=pal["body"])
    d.line([1, 21, 31, 21], fill=pal["deep"], width=1)
    # supports
    d.rectangle([5, 21, 9.5, 29], fill=pal["deep"])
    d.rectangle([22.5, 21, 27, 29], fill=pal["deep"])
    # fissures on the surface...
    cracks(d, rng, (4, 15, 26, 17), 2, pal)
    cracks(d, rng, (4, 18, 26, 20.4), 2, pal)
    # ...or carved names: two clusters of rune-like letter scratches
    for (x0, y0) in [(11, 18.6), (19, 18.8)]:
        for k in range(3):
            x = x0 + k * 2.2
            d.line([x, y0 + 1.6, x + 0.8, y0, x + 1.6, y0 + 1.6],
                   fill=pal["pale"], width=1)
    glints(d, rng, 5, box=(3, 15, 29, 20))
    return img

# ---------------------------------------------------------------- washer
def draw_washer():
    rng = random.Random(3417); pal = DARK_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    panel(d, 6, 5, 26, 29, pal)
    # control strip
    d.line([7, 9.5, 25, 9.5], fill=pal["deep"], width=1)
    d.ellipse([8.5, 6.8, 10.5, 8.8], outline=pal["pale"])
    d.rectangle([20, 7, 24, 8.6], outline=pal["facet"])
    # the porthole, completely filled with black sand
    d.ellipse([8.5, 11.5, 23.5, 26.5], fill=pal["facet"], outline=pal["pale"])
    d.ellipse([10.4, 13.4, 21.6, 24.6], fill=SAND)
    # sand surface: a settled mound line inside the glass
    d.arc([10.4, 10.5, 21.6, 21], start=0, end=180, fill=(30, 38, 40, 255),
          width=2)
    for _ in range(12):
        x = rng.uniform(11, 21); y = rng.uniform(19, 24)
        d.point((x, y), fill=(34, 42, 44, 255))
    # spilled sand on the floor in front
    d.polygon([(10, 29.5), (22, 29.5), (26, 31.5), (6, 31.5)], fill=SAND)
    for _ in range(10):
        x = rng.uniform(7, 25); y = rng.uniform(29.6, 31.3)
        d.point((x, y), fill=(40, 50, 50, 255))
    glints(d, rng, 4, box=(7, 5, 25, 10))
    return img

# ---------------------------------------------------------------- dryer
def draw_dryer():
    rng = random.Random(3418); pal = DARK_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    panel(d, 7, 5, 27, 29, pal)
    d.line([8, 9.5, 26, 9.5], fill=pal["deep"], width=1)
    d.ellipse([21.5, 6.8, 23.5, 8.8], outline=pal["pale"])
    # drum opening
    d.ellipse([11.5, 11.5, 26.5, 26.5], fill=pal["deep"], outline=pal["facet"])
    # the jumble of smashed clothes inside: angular pale shards
    shard_tones = [pal["pale"], pal["facet"], (120, 152, 148, 255),
                   pal["body2"]]
    for _ in range(9):
        x = rng.uniform(14, 23); y = rng.uniform(19, 24.5)
        w = rng.uniform(1.8, 3.4); h = rng.uniform(1.2, 2.4)
        col = shard_tones[rng.randrange(len(shard_tones))]
        d.polygon([(x, y), (x + w, y + rng.uniform(-0.6, 0.6)),
                   (x + w * 0.6, y + h), (x - w * 0.2, y + h * 0.7)],
                  fill=col)
    d.arc([11.5, 11.5, 26.5, 26.5], start=300, end=60, fill=pal["pale"],
          width=1)
    # the door, hanging open on its left hinge
    d.ellipse([1, 11, 9.5, 27], fill=pal["body2"], outline=pal["pale"])
    d.ellipse([2.8, 13.2, 7.7, 24.8], fill=pal["facet"], outline=pal["deep"])
    d.line([8.8, 13, 10.8, 12], fill=pal["hi"], width=1)  # hinge light
    glints(d, rng, 4, box=(8, 5, 26, 10))
    return img

# ---------------------------------------------------------------- bookcases
def _bookcase(rng, pal, missing_book):
    img = new_img(); d = ImageDraw.Draw(img)
    panel(d, 5, 3, 27, 29, pal)
    d.rectangle([7, 5, 25, 27], fill=pal["deep"])  # interior back
    tones = [pal["body2"], pal["facet"], (56, 74, 74, 255), (36, 48, 50, 255)]
    shelf_rows = [(5.5, 12), (12.5, 19), (19.5, 26.5)]  # (top, shelf board)
    for (top, board) in shelf_rows:
        x = 7.6
        while x < 23.6:
            w = rng.uniform(1.7, 2.7)
            h = rng.uniform(4.6, 6.2)
            if missing_book and board == 19 and 13 < x < 17:
                # the missing book: a dark slot with a shelf shadow
                d.rectangle([x, board - 5.6, x + w + 0.6, board],
                            fill=(8, 11, 13, 255))
                x += w + 1.4
                missing_book = False
                continue
            col = tones[rng.randrange(len(tones))]
            # books fused into each other: shared edges, no gaps
            d.rectangle([x, board - h, x + w, board], fill=col)
            d.line([x, board - h, x + w, board - h], fill=pal["pale"],
                   width=1)
            if rng.random() < 0.55:  # a title, in an unfamiliar alphabet
                tx = x + w / 2
                d.line([tx, board - h + 1.2, tx, board - h + 2.6],
                       fill=pal["hi"], width=1)
                d.point((tx, board - h + 3.6), fill=pal["hi"])
            x += w + rng.uniform(0, 0.35)
        d.rectangle([7, board, 25, board + 1.1], fill=pal["facet"])
        d.line([7, board, 25, board], fill=pal["pale"], width=1)
    return img

def draw_bookcase():
    return _bookcase(random.Random(3419), DARK_PAL, False)

def draw_bookcase_talker():
    return _bookcase(random.Random(3420), DARK_PAL, True)

# ---------------------------------------------------------------- counter
def draw_counter():
    rng = random.Random(3421); pal = DARK_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    # thick cube: top plane, front face, shaded right side
    d.polygon([(4, 10), (27, 10), (30, 13.5), (2, 13.5)], fill=pal["facet"])
    d.line([4, 10, 27, 10], fill=pal["pale"], width=1)
    d.polygon([(2, 13.5), (30, 13.5), (30, 28), (2, 28)], fill=pal["body"])
    d.polygon([(27, 10), (30, 13.5), (30, 28), (27, 28)], fill=pal["deep"])
    d.line([2, 28, 30, 28], fill=pal["deep"], width=1)
    # statues of utensils, fused in place on the surface: drawn bold and
    # bright so they survive at tile scale
    # fork: handle + three tines
    d.line([6.5, 12.8, 10, 11.4], fill=GLASS_SHEEN, width=2)
    for k in (-1, 0, 1):
        d.line([10, 11.4, 12.6, 10.8 + k * 0.8], fill=GLASS_SHEEN, width=1)
    # spoon: handle + oval bowl
    d.line([13.8, 12.6, 17.2, 11.6], fill=GLASS_SHEEN, width=2)
    d.ellipse([16.9, 10, 20.4, 12.2], fill=pal["pale"], outline=GLASS_SHEEN)
    # knife: dark handle + pale blade
    d.line([20.8, 12.4, 23.2, 11.8], fill=pal["deep"], width=2)
    d.polygon([(23.2, 11.8), (27.2, 10.6), (23.6, 12.8)], fill=pal["pale"],
              outline=GLASS_SHEEN)
    # a fused miscellany blob
    d.ellipse([4.6, 11, 7.4, 13], fill=pal["facet"], outline=pal["pale"])
    glints(d, rng, 6, box=(3, 14, 29, 27))
    return img

# ---------------------------------------------------------------- table
def draw_table():
    rng = random.Random(3422); pal = DARK_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    # back legs (shorter, darker), then the slab, then the front legs
    d.rectangle([7, 15, 9.5, 24], fill=pal["deep"])
    d.rectangle([22.5, 15, 25, 24], fill=pal["deep"])
    d.polygon([(2, 11.5), (30, 11.5), (31, 14.5), (1, 14.5)],
              fill=pal["facet"])
    d.line([2, 11.5, 30, 11.5], fill=pal["pale"], width=1)
    d.line([1, 14.5, 31, 14.5], fill=pal["deep"], width=1)
    d.rectangle([4, 14.5, 7.5, 29], fill=pal["body"])
    d.rectangle([24.5, 14.5, 28, 29], fill=pal["body"])
    d.line([4.8, 15, 4.8, 28.4], fill=pal["pale"], width=1)
    # shallowly rounded spill masses on the surface
    for (x, y, w) in [(11, 12.2, 5.5), (21.5, 12.6, 4.2)]:
        d.ellipse([x - w / 2, y - 0.9, x + w / 2, y + 0.9], fill=pal["pale"])
        d.arc([x - w / 2, y - 0.9, x + w / 2, y + 0.9], start=180, end=330,
              fill=pal["hi"], width=1)
    glints(d, rng, 5, box=(3, 15, 29, 28))
    return img

# ---------------------------------------------------------------- birdbath
def draw_birdbath():
    rng = random.Random(3423); pal = LIGHT_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    # pedestal: foot, column, flare
    d.polygon([(11.5, 29.5), (20.5, 29.5), (19.5, 26.5), (12.5, 26.5)],
              fill=pal["body"])
    d.rectangle([14, 16, 18, 27], fill=pal["body"])
    d.line([14.8, 16.5, 14.8, 26.5], fill=pal["hi"], width=1)
    d.rectangle([12.8, 15, 19.2, 16.6], fill=pal["facet"])
    # bowl
    d.ellipse([3, 9.5, 29, 16.5], fill=pal["facet"], outline=pal["hi"])
    d.ellipse([5.8, 11, 26.2, 15.2], fill=SAND)  # black sand in the bowl
    d.arc([5.8, 11, 26.2, 15.2], start=190, end=340, fill=(44, 58, 58, 255),
          width=1)
    # the delicate bird, frozen mid-wash: body, raised wing, head, tail
    d.ellipse([12.5, 8.6, 19, 12.2], fill=pal["pale"], outline=pal["hi"])
    d.polygon([(13.2, 9.6), (10.6, 6.2), (15, 8.2)], fill=pal["pale"],
              outline=pal["hi"])  # tail flicked up
    d.polygon([(15.5, 8.8), (17.8, 5.2), (18.6, 9.2)], fill=(150, 184, 178, 255),
              outline=pal["hi"])  # wing raised mid-splash
    d.ellipse([18.2, 6.4, 21, 9], fill=pal["pale"], outline=pal["hi"])  # head
    d.polygon([(20.8, 7.4), (23, 8), (20.8, 8.4)], fill=pal["hi"])  # beak
    d.point((19.4, 7.4), fill=pal["deep"])
    d.line([15, 12, 14.6, 13.4], fill=pal["hi"], width=1)  # legs in the sand
    d.line([16.8, 12, 17.2, 13.4], fill=pal["hi"], width=1)
    glints(d, rng, 5, box=(4, 10, 28, 29), col=pal["hi"])
    return img

# ---------------------------------------------------------------- shower
def draw_shower():
    rng = random.Random(3424); pal = DARK_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    # riser pipe and arm
    d.rectangle([5.5, 3, 7.5, 27], fill=pal["body"])
    d.line([6.2, 3, 6.2, 27], fill=pal["pale"], width=1)
    d.rectangle([5.5, 3, 16, 4.6], fill=pal["body"])
    # the head, angled down
    d.polygon([(13, 4.2), (20, 4.2), (21.5, 7.5), (11.5, 7.5)],
              fill=pal["facet"], outline=pal["pale"])
    # a dozen streams of thin black glass, droplets strung like pearls
    for k in range(12):
        x0 = 12.3 + k * 0.85
        pts = [(x0 + 0.35 * math.sin(y / 5.5 + k * 1.3), y)
               for y in range(8, 27, 2)]
        d.line(pts, fill=(52, 70, 70, 255), width=1)
        if k % 3 == 0:  # frozen droplets on some streams
            for y in (12.5, 18, 23.5):
                xx = x0 + 0.35 * math.sin(y / 5.5 + k * 1.3)
                d.ellipse([xx - 0.8, y - 1.1, xx + 0.8, y + 1.1],
                          fill=pal["pale"])
    # base tray
    d.polygon([(8, 26.5), (26, 26.5), (28, 29.5), (6, 29.5)],
              fill=pal["body"])
    d.line([8, 26.5, 26, 26.5], fill=pal["pale"], width=1)
    glints(d, rng, 4, box=(9, 27, 25, 29))
    return img

# ---------------------------------------------------------------- water heater
def draw_water_heater():
    rng = random.Random(3425); pal = DARK_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    # cylinder body with vertical facet bands
    d.rectangle([10, 7, 22, 29.5], fill=pal["body"])
    d.rectangle([10, 7, 13, 29.5], fill=pal["facet"])
    d.rectangle([19.5, 7, 22, 29.5], fill=pal["deep"])
    d.line([10.8, 8, 10.8, 29], fill=pal["pale"], width=1)
    d.line([10, 29.5, 22, 29.5], fill=pal["deep"], width=1)
    # a bent pipe stub at the top left
    d.rectangle([11, 3.5, 13, 7], fill=pal["body"])
    d.line([11.6, 3.5, 11.6, 7], fill=pal["pale"], width=1)
    # the burst top: jagged shards bent outward around the rim
    crown = [(9.5, 8), (10.5, 2.5), (12.5, 6.5), (14, 1.5), (15.5, 6),
             (17.5, 2.2), (19, 6.5), (20.5, 3), (22.5, 8)]
    d.polygon(crown + [(22, 9.5), (10, 9.5)], fill=pal["facet"])
    for (x, y) in [(14, 2.8), (17.5, 3.6), (20.5, 4.2)]:
        d.point((x, y), fill=pal["hi"])
    d.ellipse([12, 6.8, 20, 9.6], fill=SAND)  # black sand welling out
    # rivulets of black sand running down the side
    for (x, w, ln) in [(14.5, 1.6, 13), (18, 1.2, 9), (20.8, 1.4, 16)]:
        pts = [(x + 0.5 * math.sin(y / 4 + x), y) for y in range(9, 9 + ln, 2)]
        d.line(pts, fill=SAND, width=2)
        for _ in range(4):
            d.point((x + rng.uniform(-1.4, 1.4), rng.uniform(10, 9 + ln)),
                    fill=SAND)
    glints(d, rng, 5, box=(10, 10, 21, 28))
    return img

# ---------------------------------------------------------------- fireplace
def draw_fireplace():
    rng = random.Random(3426); pal = DARK_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    panel(d, 4, 5, 28, 29, pal)
    # pitted glass-brick courses
    for y in range(9, 28, 4):
        d.line([5, y, 27, y], fill=pal["deep"], width=1)
        off = 4 if (y // 4) % 2 else 0
        for x in range(5 + off, 27, 8):
            d.line([x, y, x, y + 3.4], fill=pal["deep"], width=1)
    for _ in range(26):  # pits
        x = rng.uniform(5, 27); y = rng.uniform(6, 28)
        if 10 < x < 22 and y > 14:
            continue
        d.point((x, y), fill=pal["deep"])
    # firebox opening
    d.rectangle([10.5, 16, 21.5, 29], fill=(6, 8, 10, 255))
    d.arc([10.5, 11, 21.5, 21], start=180, end=360, fill=(6, 8, 10, 255),
          width=5)
    # the frozen fire, leaping beyond the grate: opaque pale glass tongues
    tongues = [(12.5, 26, 11.2, 15.5, 14, 22), (15.5, 27, 15.8, 12.5, 17.5, 23),
               (18.8, 26, 20.8, 16.5, 19.6, 22)]
    for (x0, y0, tx, ty, x1, y1) in tongues:
        d.polygon([(x0, y0), (tx, ty), (x1, y1)], fill=pal["pale"])
    d.polygon([(14.8, 26.5), (16, 18), (17.2, 26.5)], fill=pal["hi"])
    # the grate it leaps beyond
    d.line([10.5, 24.5, 21.5, 24.5], fill=pal["hi"], width=1)
    for x in range(12, 21, 2):
        d.line([x, 23, x, 26], fill=pal["pale"], width=1)
    # hearth
    d.rectangle([7.5, 28.6, 24.5, 30], fill=pal["facet"])
    glints(d, rng, 5, box=(5, 6, 27, 13))
    return img

# ---------------------------------------------------------------- firering
def draw_firering():
    rng = random.Random(3427); pal = DARK_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    # ash disc inside
    d.ellipse([8.5, 9.5, 23.5, 24.5], fill=(10, 13, 15, 255))
    # a circle of smooth glass pebbles
    for k in range(11):
        a = k / 11 * math.tau + 0.2
        cx, cy = 16 + 10.2 * math.cos(a), 17 + 9.6 * math.sin(a)
        w = rng.uniform(3.2, 4.4); h = w * 0.78
        tone = [pal["body"], pal["body2"], pal["facet"]][rng.randrange(3)]
        d.ellipse([cx - w, cy - h, cx + w, cy + h], fill=tone)
        d.point((cx - w * 0.35, cy - h * 0.35), fill=pal["hi"])
    # frozen tendrils of fire: opaque pale cores...
    for (x0, tx, x1, top) in [(12.5, 11.5, 14, 10.5), (15.8, 16.2, 17.5, 8.5),
                              (19, 20, 20.5, 11.5)]:
        d.polygon([(x0, 21), (tx, top), (x1, 21)], fill=pal["pale"])
    # ...and their slightly translucent outer wisps, composited
    ov = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    od = ImageDraw.Draw(ov)
    for (x, top, ph) in [(13, 9, 0.5), (16.2, 7, 2.0), (19.5, 9.5, 3.6),
                         (14.8, 11, 1.2), (18, 12, 2.8)]:
        pts = [(x + 0.9 * math.sin(y / 2.6 + ph), y)
               for y in range(21, int(top), -2)]
        od.line(pts, fill=(190, 230, 220, 110), width=2)
        od.ellipse([x - 1, top - 1.6, x + 1, top + 0.6],
                   fill=(210, 240, 232, 130))
    img.alpha_composite(ov)
    return img

# ---------------------------------------------------------------- floor lamp
def draw_floor_lamp():
    rng = random.Random(3428); pal = LIGHT_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    # base and tall pole
    d.ellipse([10.5, 27.5, 21.5, 30], fill=pal["body"], outline=pal["hi"])
    d.rectangle([15.2, 9, 16.8, 28], fill=pal["facet"])
    d.line([15.6, 9, 15.6, 28], fill=pal["hi"], width=1)
    # the translucent shade: pale glass, drawn opaque...
    d.polygon([(10, 2.5), (22, 2.5), (24, 9.5), (8, 9.5)], fill=(150, 184, 178, 255))
    d.line([10, 2.5, 22, 2.5], fill=pal["hi"], width=1)
    d.line([8, 9.5, 24, 9.5], fill=pal["facet"], width=1)
    # ...with the soft glow you might almost see, composited
    ov = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    od = ImageDraw.Draw(ov)
    od.ellipse([11, 3.4, 21, 8.8], fill=(235, 248, 240, 90))
    od.ellipse([5.5, 0.5, 26.5, 12], outline=(220, 240, 232, 36), width=2)
    img.alpha_composite(ov)
    glints(d, rng, 4, box=(13, 12, 19, 27), col=pal["hi"])
    return img

# ---------------------------------------------------------------- box
def draw_box():
    rng = random.Random(3429); pal = DARK_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    # thin-walled cube: top plane, front, shaded right side
    d.polygon([(6, 7.5), (25, 7.5), (28, 11), (3, 11)], fill=pal["facet"])
    d.polygon([(3, 11), (28, 11), (28, 27), (3, 27)], fill=pal["body"])
    d.polygon([(25, 7.5), (28, 11), (28, 27), (25, 27)], fill=pal["deep"])
    # lid seam
    d.line([3, 13.5, 28, 13.5], fill=pal["deep"], width=1)
    # edges glisteningly sharp: sheen-bright rims
    d.line([6, 7.5, 25, 7.5], fill=GLASS_SHEEN, width=1)
    d.line([3, 11, 6, 7.5], fill=pal["hi"], width=1)
    d.line([28, 11, 25, 7.5], fill=pal["hi"], width=1)
    d.line([3, 11, 3, 27], fill=pal["pale"], width=1)
    for (x, y) in [(6, 7.5), (25, 7.5), (15, 9)]:
        d.point((x, y), fill=(255, 255, 250, 255))
    cracks(d, rng, (6, 16, 20, 25), 1, pal)
    glints(d, rng, 5, box=(4, 14, 27, 26))
    return img

# ---------------------------------------------------------------- sofa
def _sofa(rng, pal):
    img = new_img(); d = ImageDraw.Draw(img)
    # long back with two back cushions
    rrect(d, [4, 5.5, 28, 20], radius=3, fill=pal["body"])
    for (x0, x1) in [(7, 15), (17, 25)]:
        rrect(d, [x0, 7.5, x1, 15.5], radius=2, fill=pal["facet"])
    d.arc([7, 7.5, 15, 15.5], start=180, end=320, fill=pal["pale"], width=1)
    d.arc([17, 7.5, 25, 15.5], start=180, end=320, fill=pal["pale"], width=1)
    # arms
    for (x0, x1) in [(2, 7.5), (24.5, 30)]:
        rrect(d, [x0, 11.5, x1, 25], radius=2.5, fill=pal["body2"])
        d.line([x0 + 0.8, 11.8, x1 - 0.8, 11.8], fill=pal["pale"], width=1)
    # two cold seat cushions and the base
    for (x0, x1) in [(7, 15), (17, 25)]:
        rrect(d, [x0, 16.5, x1, 22.5], radius=2, fill=pal["facet"])
    d.rectangle([4.5, 22.5, 27.5, 26.5], fill=pal["body"])
    d.rectangle([6.5, 26.5, 9, 29], fill=pal["deep"])
    d.rectangle([23, 26.5, 25.5, 29], fill=pal["deep"])
    return img

def draw_sofa():
    rng = random.Random(3430); pal = DARK_PAL
    img = _sofa(rng, pal)
    d = ImageDraw.Draw(img)
    sheen(img, rng, 1); glints(d, rng, 6, box=(5, 7, 27, 25))
    return img

# ---------------------------------------------------------------- dishwasher
def draw_dishwasher():
    rng = random.Random(3431); pal = DARK_PAL
    img = new_img(); d = ImageDraw.Draw(img)
    panel(d, 6, 4, 26, 24, pal)
    # control strip
    d.line([7, 8, 25, 8], fill=pal["deep"], width=1)
    d.point((9, 6), fill=pal["hi"]); d.point((12, 6), fill=pal["facet"])
    d.rectangle([19, 5.4, 24, 7], outline=pal["facet"])
    # interior with a rack of plates
    d.rectangle([8.5, 10, 23.5, 22], fill=pal["deep"])
    for x in (11.5, 15, 18.5):  # plates standing in the rack
        d.ellipse([x - 0.8, 12, x + 0.8, 19.5], outline=pal["pale"], width=1)
    d.line([9, 14.5, 23, 14.5], fill=pal["facet"], width=1)  # rack rail
    for x in (20.5, 22):  # utensils
        d.line([x, 11, x, 14], fill=pal["hi"], width=1)
    # black sand burying the bottom of the interior
    d.rectangle([8.5, 18.5, 23.5, 22], fill=SAND)
    d.arc([8.5, 15.5, 23.5, 21.5], start=0, end=180, fill=SAND, width=2)
    # the door, hanging open: swung down toward the viewer
    d.polygon([(6, 24), (26, 24), (29, 29.5), (3, 29.5)], fill=pal["body2"])
    d.polygon([(8, 25.2), (24, 25.2), (26, 28.4), (6, 28.4)],
              fill=pal["facet"])
    d.line([6, 24, 26, 24], fill=pal["hi"], width=1)
    glints(d, rng, 4, box=(7, 5, 25, 8))
    return img

# ---------------------------------------------------------------- doll
def draw_doll():
    rng = random.Random(3432); pal = DARK_PAL
    img = _sofa(rng, pal)
    d = ImageDraw.Draw(img)
    # a soft plush torso lying across the seat -- matte cloth, NOT glass,
    # and empty of features: no head, no face, just the torso and stubs
    cloth = (106, 94, 84, 255); cloth_sh = (84, 74, 66, 255)
    cloth_hi = (128, 114, 100, 255)
    # stub arms angled out from the shoulders
    rrect(d, [7.5, 14.5, 11.5, 18.5], radius=1.8, fill=cloth_sh)
    rrect(d, [20.5, 14.5, 24.5, 18.5], radius=1.8, fill=cloth_sh)
    # the torso: a rounded, slightly sagging capsule
    rrect(d, [10, 13.5, 22, 20.5], radius=3.2, fill=cloth)
    d.arc([10.5, 14, 21.5, 18], start=180, end=330, fill=cloth_hi, width=1)
    d.arc([10.5, 16.5, 21.5, 20.4], start=30, end=150, fill=cloth_sh, width=1)
    # a plain stitched seam down the middle -- its only feature
    for y in range(14, 20):
        if y % 2 == 0:
            d.point((16, y), fill=cloth_sh)
    # hip stubs at the bottom corners
    rrect(d, [10.5, 19, 13.5, 21.5], radius=1.4, fill=cloth_sh)
    rrect(d, [18.5, 19, 21.5, 21.5], radius=1.4, fill=cloth_sh)
    glints(d, rng, 4, box=(5, 7, 27, 12))
    return img

# ---------------------------------------------------------------- jobs
JOBS = [
    ("f_vitrified_bed", draw_bed()),
    ("f_vitrified_bathtub", draw_bathtub()),
    ("f_vitrified_dresser", draw_dresser()),
    ("f_vitrified_fridge", draw_fridge()),
    ("f_vitrified_armchair", draw_armchair()),
    ("f_vitrified_chair", draw_chair()),
    ("f_vitrified_desk", draw_desk()),
    ("f_vitrified_locker", draw_locker()),
    ("f_vitrified_oven", draw_oven()),
    ("f_vitrified_mailbox", draw_mailbox()),
    ("f_vitrified_rack", draw_rack()),
    ("f_vitrified_sink", draw_sink()),
    ("f_vitrified_toilet", draw_toilet()),
    ("f_vitrified_cupboard", draw_cupboard()),
    ("f_vitrified_planter", draw_planter()),
    ("f_vitrified_bench", draw_bench()),
    ("f_vitrified_washer", draw_washer()),
    ("f_vitrified_dryer", draw_dryer()),
    ("f_vitrified_bookcase", draw_bookcase()),
    ("f_vitrified_counter", draw_counter()),
    ("f_vitrified_table", draw_table()),
    ("f_vitrified_birdbath", draw_birdbath()),
    ("f_vitrified_shower", draw_shower()),
    ("f_vitrified_water_heater", draw_water_heater()),
    ("f_vitrified_fireplace", draw_fireplace()),
    ("f_vitrified_firering", draw_firering()),
    ("f_vitrified_floor_lamp", draw_floor_lamp()),
    ("f_vitrified_box", draw_box()),
    ("f_vitrified_sofa", draw_sofa()),
    ("f_vitrified_dishwasher", draw_dishwasher()),
    ("f_vitrified_doll", draw_doll()),
    ("f_vitrified_bookcase_talker", draw_bookcase_talker()),
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
def tile_img(tid):
    p = os.path.join(BASE, "pngs_normal_32x32", "furniture", tid, tid + ".png")
    return Image.open(p).convert("RGBA")

CELL = 64
COLS = 6
ROWS = (len(JOBS) + COLS - 1) // COLS
sheet = Image.new("RGBA", (COLS * CELL, ROWS * CELL), (24, 24, 28, 255))
for i, (tid, _img) in enumerate(JOBS):
    t = tile_img(tid).resize((CELL, CELL), Image.NEAREST)
    sheet.alpha_composite(t, ((i % COLS) * CELL, (i // COLS) * CELL))
sheet.save(os.path.join(BASE, "preview_vitrified_furniture.png"))
print("wrote preview_vitrified_furniture.png", sheet.size)
