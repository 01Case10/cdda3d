#!/usr/bin/env python3
"""Batch ITEM32 (Phase 2g.4 scope extension): medical items.

45 ids (authoritative list embedded below; source list:
~/workspace/item-batches/32.txt). All 45 resolve to ITEM
entries in data/json/items/comestibles/med.json (birth_control
and crack also exist as an item_group / addiction_type under
the same id; the ITEM entries are authoritative here).
Copy-from chains were resolved against the data before
drawing. Every entry describes a physical object, so no
debug tiles were needed.

Families, parameterized from the entries' data:

- Pill bottles (the bottle_plastic_pill_* containers):
  white body for the painkiller/supplement bottles, amber
  for the prescription bottles; cap, label band and a
  label motif per drug, loose pills of the right shape in
  front. acetaminophen (red label, round white pills),
  ibuprofen (rust label, oval rust tablets), naproxen
  (blue label), metaprolol (teal label, heart motif),
  adderall (light-red label, capsule motif), pills_sleep
  (indigo label, moon + Z), prozac (green label, capsule),
  thorazine (crimson label, wavy brain lines), xanax
  (cyan label, scored bar pill), antihistamine (cream
  label, flower), melatonin (yellow cap, moon + star).
- Loose tablets: birth_control as the classic round
  blister dial pack; estrogen as tiny white sublingual
  tablets; poppy_pain pink tablets with a poppy motif;
  poppy_sleep magenta tablets with a moon pip; tums as a
  small bottle with chalky pastel tablets; vitc_tablet as
  white tablets with magenta sparkles; vitc_fragment as a
  broken wedge of one; panacea as itself -- one huge
  apple-red gel capsule, black-to-purple oily swirl and
  gray flecks inside, per its description.
- Contact lens cases (13): the twin-well plastic case in
  saline cyan. Parameters: Rx glyph on the case front
  (minus = myopia, plus = hyperopia, plus-over-minus =
  bifocal, plain ring = plano), transition variants get
  smoky darkened caps and a sun pip (they darken in
  light), weekly cases get a double base stripe where
  daily cases get a single one.
- Liquids in small bottles: makeshift antiseptic (pale
  cyan, scrawled label, red cross), skunk-spray
  neutralizer (white, skunk-tail label motif), cocaine
  topical cream (squat jar, white cream swirl), sunscreen
  (squeeze tube, sun motif), wintergreen oil (ointment
  tin, leaf motif), steroid eyedrops (dropper bottle,
  eye + drop motif).
- Vials: injectable iron (dark yellow liquid), prophy-
  lactic antivenom (amber vial, snake motif).
- Tobacco forms: chaw as a green foil chewing-tobacco
  pouch, rolling tobacco as a baggie of brown shreds,
  nicotine pouches as three small white pouches, crack
  as white crystals in a zipper bag.
- debug_chloroform_rag: a folded pale rag with vapor
  wisps and a small red "?" badge (its data color is
  red; it is a debug item but describes a real rag).

Glass/plastic translucency is faked with pale opaque
colors (this Pillow build replaces rather than blends);
all sprite alpha is in {0, 255}, asserted per sprite.

Writes ONLY its own 45 folders + preview_item32.png.
PIL only; per-sprite random.Random(crc32(id)).
"""
import json
import os
import random
import zlib

from PIL import Image, ImageDraw

IDS = [
    "acetaminophen", "adderall", "antihistamine",
    "bifocal_contacts_daily", "bifocal_contacts_weekly",
    "bifocal_transition_contacts_daily",
    "bifocal_transition_contacts_weekly", "birth_control",
    "chaw", "cocaine_topical", "crack", "debug_chloroform_rag",
    "disinfectant_makeshift", "estrogen",
    "hyperopia_contacts_daily", "hyperopia_contacts_weekly",
    "hyperopia_transition_contacts_daily",
    "hyperopia_transition_contacts_weekly", "ibuprofen",
    "inj_iron", "melatonin_tablet", "metaprolol",
    "myopia_contacts_daily", "myopia_transition_contacts_daily",
    "myopia_transition_contacts_weekly", "naproxen", "nic_pouch",
    "panacea", "pills_sleep", "plano_transition_contacts_daily",
    "plano_transition_contacts_weekly", "poppy_pain", "poppy_sleep",
    "prophylactic_antivenom", "prozac",
    "skunk_spray_neutralizing_solution", "steroid_eyedrops",
    "sunscreen", "thorazine", "tobacco", "tums", "vitc_fragment",
    "vitc_tablet", "wintergreen_oil", "xanax",
]

BASE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                    "pngs_normal_32x32", "items")
CREDITS = ("Drawn from the item's in-game description for the CDDA Voxel "
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, "
           "as the tileset.\n")

INK = (38, 34, 40, 255)
WHITE = (238, 238, 240, 255)
PAPER = (226, 220, 200, 255)
WHCAP = (225, 225, 228, 255)


def new_sprite():
    return Image.new("RGBA", (32, 32), (0, 0, 0, 0))


# ---------- tiny motif glyphs (drawn in a 7x7 box at x, y) ----------

def g_cross(d, x, y, c):
    d.rectangle([x + 2, y, x + 4, y + 6], fill=c)
    d.rectangle([x, y + 2, x + 6, y + 4], fill=c)


def g_heart(d, x, y, c):
    d.rectangle([x, y + 1, x + 2, y + 3], fill=c)
    d.rectangle([x + 4, y + 1, x + 6, y + 3], fill=c)
    d.rectangle([x, y + 2, x + 6, y + 3], fill=c)
    d.polygon([(x, y + 3), (x + 6, y + 3), (x + 3, y + 6)], fill=c)


def g_moon(d, x, y, c):
    d.ellipse([x + 1, y, x + 6, y + 6], fill=c)
    d.ellipse([x + 3, y, x + 7, y + 5], fill=PAPER)


def g_star(d, x, y, c):
    d.point((x + 3, y), fill=c)
    d.rectangle([x + 2, y + 1, x + 4, y + 5], fill=c)
    d.rectangle([x, y + 2, x + 6, y + 3], fill=c)
    d.point((x + 3, y + 6), fill=c)


def g_flower(d, x, y, c):
    for dx, dy in ((3, 0), (0, 2), (6, 2), (1, 5), (5, 5)):
        d.ellipse([x + dx - 1, y + dy - 1, x + dx + 1, y + dy + 1], fill=c)
    d.point((x + 3, y + 2), fill=INK)


def g_capsule(d, x, y, c1, c2):
    d.ellipse([x, y + 1, x + 3, y + 5], fill=c1)
    d.rectangle([x + 2, y + 1, x + 3, y + 5], fill=c1)
    d.ellipse([x + 3, y + 1, x + 6, y + 5], fill=c2)
    d.rectangle([x + 3, y + 1, x + 4, y + 5], fill=c2)


def g_bar(d, x, y, c):
    d.rectangle([x, y + 2, x + 6, y + 4], fill=c)
    d.line([x + 2, y + 2, x + 2, y + 4], fill=INK)
    d.line([x + 4, y + 2, x + 4, y + 4], fill=INK)


def g_waves(d, x, y, c):
    for r in (0, 3):
        for i in range(7):
            yy = y + r + (1 if i % 4 in (1, 2) else 0)
            d.point((x + i, yy), fill=c)


def g_sun(d, x, y, c):
    d.ellipse([x + 2, y + 2, x + 4, y + 4], fill=c)
    for dx, dy in ((3, 0), (3, 6), (0, 3), (6, 3), (1, 1), (5, 1),
                   (1, 5), (5, 5)):
        d.point((x + dx, y + dy), fill=c)


def g_leaf(d, x, y, c):
    d.ellipse([x + 1, y, x + 6, y + 5], fill=c)
    d.line([x + 1, y + 6, x + 5, y + 1], fill=INK)


def g_eye_drop(d, x, y, c):
    d.arc([x, y + 1, x + 6, y + 5], 0, 360, fill=c)
    d.polygon([(x + 3, y + 5), (x + 2, y + 7), (x + 4, y + 7)], fill=c)
    d.point((x + 3, y + 3), fill=c)


def g_snake(d, x, y, c):
    pts = [(x + 1, y + 6), (x + 1, y + 2), (x + 3, y), (x + 5, y + 2),
           (x + 5, y + 4)]
    d.line(pts, fill=c, width=2)
    d.point((x + 5, y + 1), fill=(200, 40, 40, 255))


def g_skunk(d, x, y, c):
    d.ellipse([x + 1, y + 2, x + 5, y + 6], fill=c)
    d.rectangle([x + 2, y, x + 3, y + 6], fill=WHITE)
    d.point((x + 5, y + 1), fill=c)


def g_poppy(d, x, y, c):
    for dx, dy in ((1, 1), (4, 0), (5, 3), (2, 4)):
        d.ellipse([x + dx, y + dy, x + dx + 2, y + dy + 2], fill=c)
    d.point((x + 3, y + 2), fill=INK)
    d.line([x + 3, y + 4, x + 3, y + 6], fill=(60, 120, 50, 255))


# ---------- families ----------

def pill_bottle(rng, body, cap, label, motif=None, loose=None):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    d.rectangle([11, 3, 20, 6], fill=cap)                 # cap
    d.rectangle([11, 3, 20, 4], fill=tuple(max(0, v - 30) for v in cap[:3]) + (255,))
    d.rectangle([12, 7, 19, 8], fill=(200, 200, 205, 255))  # neck
    d.rectangle([9, 9, 22, 27], fill=body)                  # body
    d.rectangle([9, 9, 10, 27], fill=tuple(max(0, v - 25) for v in body[:3]) + (255,))
    d.rectangle([9, 13, 22, 23], fill=PAPER)                # label
    d.rectangle([9, 13, 22, 15], fill=label)
    if motif:
        motif(d, 12, 16, label)
    if loose:
        kind, c = loose
        if kind == "round":
            d.ellipse([4, 24, 8, 28], fill=c)
            d.ellipse([23, 25, 27, 29], fill=c)
            d.line([4, 26, 8, 26], fill=INK)
        elif kind == "oval":
            d.ellipse([3, 25, 9, 28], fill=c)
            d.ellipse([23, 24, 29, 27], fill=c)
        elif kind == "capsule":
            d.ellipse([3, 25, 6, 28], fill=c)
            d.ellipse([6, 25, 9, 28], fill=(240, 240, 240, 255))
            d.ellipse([23, 25, 26, 28], fill=(240, 240, 240, 255))
            d.ellipse([26, 25, 29, 28], fill=c)
    alphas = set(img.getchannel("A").getdata())
    assert alphas <= {0, 255}, alphas
    return img


def contact_case(rng, rx, transition, weekly):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    case = (196, 228, 232, 255)
    well = (150, 200, 210, 255)
    lens = (120, 180, 225, 255)
    cap_c = (105, 98, 122, 255) if transition else (168, 216, 222, 255)
    d.rectangle([4, 11, 27, 25], fill=case)
    d.rectangle([4, 11, 27, 12], fill=(225, 245, 247, 255))
    d.rectangle([4, 24, 27, 25], fill=(170, 205, 210, 255))
    for cx in (11, 20):
        d.ellipse([cx - 4, 13, cx + 4, 21], fill=cap_c)
        d.ellipse([cx - 2, 15, cx + 2, 19], fill=lens)
        d.point((cx, 17), fill=(230, 248, 250, 255))
    d.point((24, 14), fill=(200, 60, 60, 255))              # R marker
    if transition:
        g_sun(d, 13, 4, (240, 190, 60, 255))
    # Rx glyph centered low on the case front
    gx, gy = 13, 26
    if rx in ("minus", "both"):
        d.rectangle([gx, gy + 1, gx + 5, gy + 2], fill=INK)
    if rx in ("plus", "both"):
        yy = gy - 4 if rx == "both" else gy
        d.rectangle([gx, yy + 1, gx + 5, yy + 2], fill=INK)
        d.rectangle([gx + 2, yy, gx + 3, yy + 3], fill=INK)
    if rx == "plano":
        d.arc([gx + 1, gy - 1, gx + 5, gy + 3], 0, 360, fill=INK)
    # daily = one stripe, weekly = two
    d.rectangle([6, 28, 25, 28], fill=well)
    if weekly:
        d.rectangle([6, 30, 25, 30], fill=well)
    alphas = set(img.getchannel("A").getdata())
    assert alphas <= {0, 255}, alphas
    return img


def small_bottle(rng, liquid, motif=None, scrawl=False, cap=(90, 90, 98, 255)):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    d.rectangle([13, 3, 18, 6], fill=cap)
    d.rectangle([14, 7, 17, 8], fill=(200, 200, 205, 255))
    d.rectangle([10, 9, 21, 27], fill=(214, 226, 230, 255))
    d.rectangle([11, 15, 20, 26], fill=liquid)
    d.rectangle([10, 10, 21, 13], fill=PAPER)
    if scrawl:
        g_waves(d, 12, 10, (90, 90, 90, 255))
        g_cross(d, 18, 17, (200, 50, 45, 255))
    elif motif:
        motif(d, 12, 10, INK)
    alphas = set(img.getchannel("A").getdata())
    assert alphas <= {0, 255}, alphas
    return img


def vial(rng, liquid, motif=None):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    d.rectangle([12, 4, 19, 7], fill=(150, 150, 158, 255))
    d.rectangle([13, 8, 18, 9], fill=(190, 190, 195, 255))
    d.rectangle([11, 10, 20, 27], fill=(206, 218, 222, 255))
    d.rectangle([12, 14, 19, 26], fill=liquid)
    d.rectangle([11, 17, 20, 23], fill=PAPER)
    if motif:
        motif(d, 12, 18, INK)
    # syringe beside the vial
    d.line([25, 8, 25, 20], fill=(200, 200, 205, 255), width=2)
    d.line([25, 20, 25, 25], fill=(160, 160, 165, 255))
    d.rectangle([23, 5, 27, 8], fill=(200, 200, 205, 255))
    alphas = set(img.getchannel("A").getdata())
    assert alphas <= {0, 255}, alphas
    return img


def tablets(rng, color, n=3, moon=False, sparkle=None):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    spots = [(8, 12), (17, 10), (13, 19)][:n]
    for (x, y) in spots:
        d.ellipse([x, y, x + 8, y + 8], fill=color)
        d.line([x, y + 4, x + 8, y + 4], fill=tuple(max(0, v - 40) for v in color[:3]) + (255,))
        d.point((x + 2, y + 2), fill=(255, 255, 255, 255))
    if moon:
        g_moon(d, 22, 20, (70, 70, 140, 255))
    if sparkle:
        for _ in range(5):
            d.point((rng.randrange(4, 28), rng.randrange(4, 28)), fill=sparkle)
    alphas = set(img.getchannel("A").getdata())
    assert alphas <= {0, 255}, alphas
    return img


def baggie(rng, fill_fn, zip_color=(210, 60, 60, 255)):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    d.rectangle([8, 6, 23, 28], fill=(205, 218, 224, 255))
    d.rectangle([8, 6, 23, 8], fill=zip_color)
    fill_fn(d, rng)
    d.rectangle([8, 6, 8, 28], fill=(235, 245, 248, 255))
    alphas = set(img.getchannel("A").getdata())
    assert alphas <= {0, 255}, alphas
    return img


BUILDERS = {}


def build(item_id):
    rng = random.Random(zlib.crc32(item_id.encode()))
    return BUILDERS[item_id](rng)


def _reg(name, fn):
    BUILDERS[name] = fn


# pill bottles
_reg("acetaminophen", lambda r: pill_bottle(r, WHITE, WHCAP, (198, 52, 48, 255), loose=("round", WHITE)))
_reg("ibuprofen", lambda r: pill_bottle(r, WHITE, WHCAP, (196, 108, 44, 255), loose=("oval", (178, 96, 52, 255))))
_reg("naproxen", lambda r: pill_bottle(r, WHITE, WHCAP, (52, 92, 178, 255), loose=("oval", WHITE)))
_reg("metaprolol", lambda r: pill_bottle(r, WHITE, WHCAP, (44, 150, 150, 255), motif=lambda d, x, y, c: g_heart(d, x, y, (198, 52, 48, 255))))
_reg("adderall", lambda r: pill_bottle(r, (196, 128, 44, 255), WHCAP, (232, 130, 130, 255), loose=("capsule", (232, 130, 130, 255))))
_reg("pills_sleep", lambda r: pill_bottle(r, (196, 128, 44, 255), WHCAP, (66, 66, 140, 255), motif=lambda d, x, y, c: g_moon(d, x, y, (240, 220, 120, 255))))
_reg("prozac", lambda r: pill_bottle(r, (196, 128, 44, 255), WHCAP, (110, 190, 90, 255), motif=lambda d, x, y, c: g_capsule(d, x, y, (110, 190, 90, 255), WHITE)))
_reg("thorazine", lambda r: pill_bottle(r, (196, 128, 44, 255), WHCAP, (168, 40, 52, 255), motif=lambda d, x, y, c: g_waves(d, x, y, (168, 40, 52, 255))))
_reg("xanax", lambda r: pill_bottle(r, (196, 128, 44, 255), WHCAP, (70, 180, 200, 255), motif=lambda d, x, y, c: g_bar(d, x, y, (200, 230, 235, 255))))
_reg("antihistamine", lambda r: pill_bottle(r, WHITE, (232, 130, 130, 255), (240, 226, 190, 255), motif=lambda d, x, y, c: g_flower(d, x, y, (220, 130, 170, 255))))
_reg("melatonin_tablet", lambda r: pill_bottle(r, WHITE, (240, 205, 70, 255), (66, 66, 140, 255), motif=lambda d, x, y, c: g_star(d, x, y, (240, 220, 120, 255))))


def _panacea(rng):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    # one huge diagonal gel capsule
    d.ellipse([5, 10, 14, 21], fill=(206, 44, 40, 255))
    d.ellipse([17, 10, 27, 21], fill=(178, 32, 34, 255))
    d.rectangle([10, 10, 22, 21], fill=(196, 40, 38, 255))
    # oily black-to-purple swirl inside
    d.ellipse([9, 13, 16, 19], fill=(30, 20, 40, 255))
    d.ellipse([16, 12, 23, 18], fill=(96, 40, 130, 255))
    for _ in range(7):  # gray flecks
        d.point((rng.randrange(8, 25), rng.randrange(12, 20)), fill=(150, 150, 150, 255))
    d.rectangle([8, 11, 13, 13], fill=(240, 130, 120, 255))  # highlight
    alphas = set(img.getchannel("A").getdata())
    assert alphas <= {0, 255}, alphas
    return img


_reg("panacea", _panacea)


def _birth_control(rng):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    d.ellipse([5, 5, 27, 27], fill=(232, 196, 208, 255))
    d.ellipse([7, 7, 25, 25], fill=(246, 232, 238, 255))
    import math
    for k in range(12):
        a = k * math.pi / 6
        cx = 16 + int(7.5 * math.cos(a))
        cy = 16 + int(7.5 * math.sin(a))
        d.ellipse([cx - 1, cy - 1, cx + 1, cy + 1], fill=(240, 214, 120, 255))
    d.ellipse([14, 14, 18, 18], fill=(200, 160, 175, 255))
    alphas = set(img.getchannel("A").getdata())
    assert alphas <= {0, 255}, alphas
    return img


_reg("birth_control", _birth_control)
_reg("estrogen", lambda r: tablets(r, (242, 240, 236, 255), n=3))
_reg("poppy_pain", lambda r: _poppy(r, (240, 150, 180, 255)))
_reg("poppy_sleep", lambda r: tablets(r, (205, 80, 190, 255), n=3, moon=True))
_reg("vitc_tablet", lambda r: tablets(r, (245, 245, 245, 255), n=3, sparkle=(225, 90, 200, 255)))


def _poppy(rng, color):
    img = tablets(rng, color, n=2)
    d = ImageDraw.Draw(img)
    g_poppy(d, 21, 20, (216, 70, 80, 255))
    return img


def _vitc_fragment(rng):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    d.polygon([(8, 10), (24, 10), (24, 16), (14, 22), (8, 18)], fill=(245, 245, 245, 255))
    d.line([8, 10, 24, 10], fill=(225, 90, 200, 255))
    d.polygon([(18, 20), (23, 21), (20, 25)], fill=(238, 238, 238, 255))
    d.point((12, 13), fill=(225, 90, 200, 255))
    alphas = set(img.getchannel("A").getdata())
    assert alphas <= {0, 255}, alphas
    return img


_reg("vitc_fragment", _vitc_fragment)


def _tums(rng):
    img = pill_bottle(rng, WHITE, (120, 170, 220, 255), (120, 170, 220, 255))
    d = ImageDraw.Draw(img)
    for (x, y, c) in ((3, 25, (240, 200, 200, 255)), (24, 24, (200, 230, 200, 255)),
                      (24, 27, (250, 240, 200, 255))):
        d.ellipse([x, y, x + 5, y + 3], fill=c)
    return img


_reg("tums", _tums)

# contact lens cases
for _rx, _names in (
    ("both", ("bifocal_contacts_daily", "bifocal_contacts_weekly",
              "bifocal_transition_contacts_daily",
              "bifocal_transition_contacts_weekly")),
    ("plus", ("hyperopia_contacts_daily", "hyperopia_contacts_weekly",
              "hyperopia_transition_contacts_daily",
              "hyperopia_transition_contacts_weekly")),
    ("minus", ("myopia_contacts_daily", "myopia_transition_contacts_daily",
               "myopia_transition_contacts_weekly")),
    ("plano", ("plano_transition_contacts_daily",
               "plano_transition_contacts_weekly")),
):
    for _n in _names:
        _tr = "transition" in _n
        _wk = _n.endswith("weekly")
        _reg(_n, (lambda rx, tr, wk: lambda r: contact_case(r, rx, tr, wk))(_rx, _tr, _wk))

# liquids and small containers
_reg("disinfectant_makeshift", lambda r: small_bottle(r, (168, 226, 232, 255), scrawl=True, cap=(150, 130, 90, 255)))
_reg("skunk_spray_neutralizing_solution", lambda r: small_bottle(r, (240, 240, 238, 255), motif=lambda d, x, y, c: g_skunk(d, x, y, (40, 40, 44, 255))))
_reg("steroid_eyedrops", lambda r: small_bottle(r, (170, 205, 240, 255), motif=lambda d, x, y, c: g_eye_drop(d, x, y, (60, 110, 190, 255)), cap=(240, 240, 240, 255)))


def _cream_jar(rng):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    d.rectangle([9, 8, 22, 11], fill=(170, 170, 176, 255))
    d.rectangle([8, 12, 23, 26], fill=(216, 220, 224, 255))
    d.rectangle([8, 16, 23, 22], fill=PAPER)
    d.ellipse([11, 4, 20, 9], fill=(248, 248, 248, 255))  # cream swirl
    d.point((15, 6), fill=(230, 230, 230, 255))
    alphas = set(img.getchannel("A").getdata())
    assert alphas <= {0, 255}, alphas
    return img


_reg("cocaine_topical", _cream_jar)


def _sunscreen(rng):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    d.rectangle([12, 4, 19, 7], fill=(240, 150, 50, 255))
    d.polygon([(11, 8), (20, 8), (22, 26), (9, 26)], fill=(246, 240, 228, 255))
    d.rectangle([11, 12, 20, 20], fill=(250, 196, 90, 255))
    g_sun(d, 12, 13, (225, 110, 30, 255))
    alphas = set(img.getchannel("A").getdata())
    assert alphas <= {0, 255}, alphas
    return img


_reg("sunscreen", _sunscreen)


def _wintergreen(rng):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    d.ellipse([7, 10, 24, 27], fill=(120, 140, 90, 255))
    d.ellipse([7, 10, 24, 15], fill=(240, 225, 130, 255))  # ointment surface
    d.ellipse([9, 11, 22, 14], fill=(250, 240, 170, 255))
    g_leaf(d, 12, 18, (46, 96, 40, 255))
    alphas = set(img.getchannel("A").getdata())
    assert alphas <= {0, 255}, alphas
    return img


_reg("wintergreen_oil", _wintergreen)

# vials
_reg("inj_iron", lambda r: vial(r, (196, 140, 26, 255)))
_reg("prophylactic_antivenom", lambda r: vial(r, (206, 138, 44, 255), motif=lambda d, x, y, c: g_snake(d, x, y, (90, 120, 50, 255))))


def _chaw(rng):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    d.rectangle([8, 7, 23, 27], fill=(64, 128, 62, 255))
    d.rectangle([8, 7, 23, 10], fill=(52, 106, 50, 255))  # crimped top
    for x in range(9, 23, 3):
        d.line([x, 7, x, 10], fill=(40, 84, 38, 255))
    d.rectangle([10, 14, 21, 23], fill=(226, 220, 190, 255))
    g_leaf(d, 12, 15, (52, 110, 46, 255))
    alphas = set(img.getchannel("A").getdata())
    assert alphas <= {0, 255}, alphas
    return img


_reg("chaw", _chaw)


def _tobacco_fill(d, rng):
    for _ in range(46):
        x = rng.randrange(10, 21)
        y = rng.randrange(11, 26)
        d.line([x, y, x + rng.randrange(1, 4), y + 1], fill=(122, 84, 44, 255))
    for _ in range(14):
        x = rng.randrange(10, 21)
        y = rng.randrange(11, 26)
        d.point((x, y), fill=(160, 118, 66, 255))


_reg("tobacco", lambda r: baggie(r, _tobacco_fill))


def _crack_fill(d, rng):
    for (x, y, s) in ((11, 13, 5), (17, 12, 4), (13, 19, 6), (19, 20, 4), (11, 23, 3)):
        d.polygon([(x, y + s), (x + 2, y), (x + s, y + 2), (x + s - 1, y + s)],
                  fill=(246, 246, 248, 255))
        d.point((x + 2, y + 2), fill=(255, 255, 255, 255))


_reg("crack", lambda r: baggie(r, _crack_fill))


def _nic_pouch(rng):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    for (x, y) in ((7, 12), (14, 9), (15, 18)):
        d.rectangle([x, y, x + 9, y + 7], fill=(242, 240, 234, 255))
        d.rectangle([x, y, x + 9, y + 1], fill=(222, 218, 208, 255))
        d.point((x + 4, y + 4), fill=(210, 205, 195, 255))
    alphas = set(img.getchannel("A").getdata())
    assert alphas <= {0, 255}, alphas
    return img


_reg("nic_pouch", _nic_pouch)


def _rag(rng):
    img = new_sprite()
    d = ImageDraw.Draw(img)
    d.polygon([(7, 14), (24, 12), (26, 24), (9, 27)], fill=(216, 212, 206, 255))
    d.polygon([(7, 14), (24, 12), (25, 17), (8, 19)], fill=(228, 224, 218, 255))
    d.line([10, 21, 23, 19], fill=(196, 192, 186, 255))
    for x in (11, 16, 21):  # vapor wisps
        d.point((x, 9), fill=(170, 200, 190, 255))
        d.point((x + 1, 7), fill=(170, 200, 190, 255))
        d.point((x, 5), fill=(170, 200, 190, 255))
    # small red "?" debug badge
    d.rectangle([24, 22, 30, 28], fill=(198, 52, 48, 255))
    d.point((27, 23), fill=WHITE)
    d.rectangle([26, 24, 28, 25], fill=WHITE)
    d.point((27, 27), fill=WHITE)
    alphas = set(img.getchannel("A").getdata())
    assert alphas <= {0, 255}, alphas
    return img


_reg("debug_chloroform_rag", _rag)


def main():
    assert len(IDS) == 45, len(IDS)
    assert set(IDS) == set(BUILDERS), set(IDS) ^ set(BUILDERS)
    for item_id in IDS:
        img = build(item_id)
        assert img.size == (32, 32)
        folder = os.path.join(BASE, item_id)
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(folder, item_id + ".png"))
        with open(os.path.join(folder, item_id + ".json"), "w") as f:
            json.dump([{"id": item_id, "fg": item_id, "bg": ""}], f, indent=2)
            f.write("\n")
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS)
    cols = 8
    rows = (len(IDS) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * 36, rows * 36), (24, 24, 28, 255))
    for i, item_id in enumerate(IDS):
        img = build(item_id)
        sheet.alpha_composite(img, ((i % cols) * 36 + 2, (i // cols) * 36 + 2))
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                       "preview_item32.png")
    sheet.save(out)
    print(f"wrote {len(IDS)} sprites + preview_item32.png {sheet.size}")


if __name__ == "__main__":
    main()
