#!/usr/bin/env python3
"""
Draw the missing mammal / bird / mutant-mammal sprites from descriptions.

Sources (fork-src/cdda/data/json/monsters/{mammal,bird,mutant_mammal}.json,
CDDA 6a3e59e0): 46 monster ids present in the coverage audit's
missing_shipped (category "monster") with no folder yet -- 17 mammals,
16 birds, 13 mutant mammals.

Sheet placement is by resolved volume (copy-from chain): every one of the
46 resolves to <100 L (the largest are the corpse placeholders via
mon_zombie_base at 62.5 L, the Mauler/Thrasher at 50 L, the Loper at
40 L), so all 46 go on the normal 32x32 sheet.

Design notes, per group (all from the entries' own descriptions):
  Birds: the chickadee gets its black cap + bib and white cheeks; the
  cedar waxwing its swept crest, black eye mask, yellow belly wash and
  tail tip and the red waxy wing dot; the guineafowl its white-spotted
  grey plumage, bare head and bony helmet. The 13 chicks resolve through
  the generic chick abstracts (tiny = dull grey, waterfowl = yellow and
  brown, generic = dark brown, chicken = yellow) with species hints on
  top: the grebe chick's striped head and back, the coot chick's dark
  down and orange-red head, the moorhen's red bill base, the duckling's
  flat bill, the scruffy big-billed pigeon squab.
  Mammals: beaver kit with paddle tail and incisors; the black cat all
  black with the described prominent green eyes; the ferret's long low
  body and dark mask; the mole's velvet cylinder body, pinpoint eyes
  and huge pink digging paws; the porcupine's banded quills; the
  skunk's white dorsal stripe and plumed tail; rabbit/hare kits with
  cotton tails. mon_human / mon_child / mon_human_scorched are corpse
  placeholders in the data ("If you see this, it's a bug"), drawn as
  plain prone figures in corpse-pallor / charred tones, no gore.
  Mutants: the deformity is the silhouette -- the bald beaver kit's
  plucked-chicken pink skin, the dambreaker kit's back bristles, the
  shark pup's too-wide toothy grin, the venom pup's slavering drips,
  the Loper's elongated cord-furred legs and reflective eyes, the
  Mauler's widened jaw crowded with teeth, the Thrasher's oversized
  digging paws and leathery bare skin, the wyrmhound's furless scaled
  serpentine head with yellow eyes and fangs, and the dextrous raccoon
  standing on its hindlimbs with long fingered paws.
Output is UltiCa decomposed layout (png + tile-entry json per folder).
"""
from PIL import Image, ImageDraw
import json, math, os, random, zlib

BASE = os.path.dirname(os.path.abspath(__file__))
CREDITS = ("Drawn from the monster's in-game description for the CDDA Voxel\n"
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.\n")
SHEET = "pngs_normal_32x32/monsters"

def rng_for(mid):
    return random.Random(zlib.crc32(mid.encode()))

def shade(c, f):
    return tuple(max(0, min(255, int(v * f))) for v in c[:3]) + (255,)

def ell(d, S, cx, cy, rx, ry, fill):
    d.ellipse([(cx - rx) * S, (cy - ry) * S, (cx + rx) * S, (cy + ry) * S], fill=fill)

def ello(d, S, cx, cy, rx, ry, dark, fill):
    o = 1.3 / S
    ell(d, S, cx, cy, rx, ry, dark)
    ell(d, S, cx, cy, max(rx - o, 0.01), max(ry - o, 0.01), fill)

def poly(d, S, pts, fill):
    d.polygon([(x * S, y * S) for x, y in pts], fill=fill)

def line(d, S, pts, fill, w=1):
    d.line([(x * S, y * S) for x, y in pts], fill=fill, width=max(1, w))

def wline(S):
    return max(1, S // 32)

# ---------------------------------------------------------------- quadrupeds
def _leg(d, S, x, top, ground, dark, fill, w=0.055, paw_r=None, paw_col=None):
    # tapered leg: wider at the hip, narrower at the ankle, dark outline
    tw, bw = w * 0.78, w * 0.52
    poly(d, S, [(x - tw, top), (x + tw, top), (x + bw, ground - 0.012),
                (x - bw, ground - 0.012)], dark)
    o = 1.2 / S
    poly(d, S, [(x - tw + o, top), (x + tw - o, top), (x + bw - o * 0.5, ground - 0.014),
                (x - bw + o * 0.5, ground - 0.014)], fill)
    ell(d, S, x + 0.016, ground - 0.006, paw_r or 0.030, 0.015, paw_col or fill)

def quadruped(S, rng, body, dark, *, bulk=1.0, leg_h=0.30, belly=None,
              tail=None, head=None, mark=None, ground=0.90, long_body=1.0,
              leg_col=None, paw_r=None):
    """Profile quadruped, head to the right. tail/head/mark are callbacks."""
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    hip = ground - leg_h
    bcx, bcy = 0.43, hip - 0.088 * bulk
    brx, bry = 0.29 * bulk * long_body, 0.105 * bulk
    geo = (bcx, bcy, brx, bry)
    if tail:
        tail(d, S, rng, bcx - brx * 0.90, bcy - bry * 0.30, dark, body)
    lc = leg_col or body
    for lx in (bcx - brx * 0.58, bcx + brx * 0.58):  # far legs, darker
        _leg(d, S, lx + 0.022, hip - 0.01, ground, dark, shade(lc, 0.78),
             w=0.062, paw_r=paw_r)
    ello(d, S, bcx, bcy, brx, bry, dark, body)
    if belly:
        ell(d, S, bcx, bcy + bry * 0.36, brx * 0.84, bry * 0.52, belly)
    for lx in (bcx - brx * 0.66, bcx + brx * 0.66):  # near legs
        _leg(d, S, lx, hip, ground, dark, lc, w=0.068, paw_r=paw_r)
    if mark:
        mark(d, S, rng, geo, dark, body)
    hx, hy = bcx + brx * 0.99, bcy - 0.05
    if head:
        head(d, S, rng, hx, hy, dark, body)
    return img

# --- tails (drawn behind the body; origin = rear of body) ---
def tail_bushy(col):
    def f(d, S, rng, x, y, dark, body):
        poly(d, S, [(x + 0.02, y + 0.06), (x - 0.10, y - 0.10), (x - 0.13, y - 0.02),
                    (x - 0.08, y + 0.10), (x + 0.03, y + 0.12)], dark)
        poly(d, S, [(x + 0.01, y + 0.05), (x - 0.085, y - 0.075), (x - 0.105, y - 0.01),
                    (x - 0.06, y + 0.085), (x + 0.02, y + 0.10)], col)
    return f

def tail_flat_beaver(d, S, rng, x, y, dark, body):
    ell(d, S, x - 0.10, y + 0.135, 0.118, 0.042, dark)
    ell(d, S, x - 0.10, y + 0.135, 0.104, 0.032, (74, 54, 40, 255))
    for t in (-0.5, 0.0, 0.5):  # paddle cross-hatch
        line(d, S, [(x - 0.10 + t * 0.10, y + 0.112), (x - 0.10 + t * 0.10, y + 0.158)],
             (52, 38, 28, 255), w=wline(S))

def tail_thin(col=None):
    def f(d, S, rng, x, y, dark, body):
        c = col or dark
        pts = [(x + 0.02, y + 0.05), (x - 0.09, y + 0.10), (x - 0.16, y + 0.085)]
        line(d, S, pts, c, w=wline(S) + 1)
    return f

def tail_cotton(d, S, rng, x, y, dark, body):
    ello(d, S, x - 0.015, y - 0.01, 0.038, 0.034, dark, (246, 244, 238, 255))

def tail_cat_up(d, S, rng, x, y, dark, body):
    pts = [(x + 0.03, y + 0.02), (x - 0.06, y - 0.06), (x - 0.075, y - 0.20)]
    line(d, S, pts, dark, w=wline(S) + 2)
    line(d, S, pts, body, w=wline(S))

def tail_dog_wag(d, S, rng, x, y, dark, body):
    pts = [(x + 0.03, y + 0.0), (x - 0.07, y - 0.09), (x - 0.13, y - 0.10)]
    line(d, S, pts, dark, w=wline(S) + 2)
    line(d, S, pts, body, w=wline(S))

def tail_long_low(d, S, rng, x, y, dark, body):  # ferret: long, low, thick
    pts = [(x + 0.02, y + 0.06), (x - 0.10, y + 0.13), (x - 0.19, y + 0.14)]
    line(d, S, pts, dark, w=wline(S) + 2)
    line(d, S, pts, body, w=wline(S) + 1)

# --- heads (hx, hy = head centre) ---
def head_dog(snout=0.105, ear="point", eye=(20, 16, 12, 255), jaw=None,
             teeth=False, grin=False, drool=False):
    def f(d, S, rng, hx, hy, dark, body):
        r = 0.082
        if ear == "point":
            poly(d, S, [(hx - 0.045, hy - 0.05), (hx - 0.055, hy - 0.125),
                        (hx + 0.005, hy - 0.075)], dark)
            poly(d, S, [(hx - 0.040, hy - 0.055), (hx - 0.046, hy - 0.105),
                        (hx - 0.002, hy - 0.070)], shade(body, 0.85))
        elif ear == "flop":
            ello(d, S, hx - 0.045, hy - 0.01, 0.030, 0.062, dark, shade(body, 0.82))
        ello(d, S, hx, hy, r, r * 0.92, dark, body)
        # snout
        sw = snout * (1.35 if grin else 1.0)
        poly(d, S, [(hx + 0.03, hy - 0.035), (hx + 0.03 + sw, hy - 0.005),
                    (hx + 0.03 + sw, hy + 0.048), (hx + 0.03, hy + 0.055)], dark)
        poly(d, S, [(hx + 0.035, hy - 0.026), (hx + 0.028 + sw, hy - 0.001),
                    (hx + 0.028 + sw, hy + 0.040), (hx + 0.035, hy + 0.046)],
             shade(body, 0.94))
        ell(d, S, hx + 0.035 + sw, hy + 0.004, 0.016, 0.013, (24, 18, 16, 255))  # nose
        if jaw:  # widened Mauler jaw
            poly(d, S, [(hx + 0.005, hy + 0.045), (hx + 0.045 + sw, hy + 0.052),
                        (hx + 0.040 + sw, hy + 0.095), (hx + 0.0, hy + 0.085)], dark)
            poly(d, S, [(hx + 0.010, hy + 0.052), (hx + 0.040 + sw, hy + 0.058),
                        (hx + 0.036 + sw, hy + 0.086), (hx + 0.004, hy + 0.078)],
                 shade(body, 0.88))
        if teeth or grin:  # tooth row along the mouth line
            n = 6 if grin else 4
            for i in range(n):
                tx = hx + 0.045 + (sw - 0.01) * (i / max(n - 1, 1))
                poly(d, S, [(tx, hy + 0.048), (tx + 0.011, hy + 0.048),
                            (tx + 0.005, hy + 0.068)], (248, 246, 238, 255))
        if drool:  # slavering strands from the mouth
            for i, dx in enumerate((0.05, 0.085, 0.12)):
                ln = 0.045 + 0.02 * ((i * 7) % 3) / 2.0
                line(d, S, [(hx + dx, hy + 0.05), (hx + dx, hy + 0.05 + ln)],
                     (196, 224, 170, 255), w=wline(S))
                ell(d, S, hx + dx, hy + 0.055 + ln, 0.008, 0.010, (196, 224, 170, 255))
        er = max(1.3, S * 0.021)  # eye
        ell(d, S, hx + 0.028, hy - 0.018, er / S, er / S, eye)
    return f

def head_cat(eye=(46, 140, 60, 255), kitten=False):
    def f(d, S, rng, hx, hy, dark, body):
        r = 0.088 if kitten else 0.076
        for ex in (hx - 0.048, hx + 0.018):
            poly(d, S, [(ex, hy - 0.045), (ex + 0.008, hy - 0.115),
                        (ex + 0.042, hy - 0.052)], dark)
            poly(d, S, [(ex + 0.006, hy - 0.050), (ex + 0.011, hy - 0.096),
                        (ex + 0.034, hy - 0.054)], shade(body, 0.8))
        ello(d, S, hx, hy, r, r * 0.9, dark, body)
        poly(d, S, [(hx + 0.03, hy + 0.01), (hx + 0.085, hy + 0.028),
                    (hx + 0.03, hy + 0.045)], shade(body, 0.92))  # short muzzle
        ell(d, S, hx + 0.082, hy + 0.026, 0.010, 0.008, (30, 22, 24, 255))
        er = max(1.4, S * 0.022)  # the prominent eye
        ell(d, S, hx + 0.030, hy - 0.012, er / S, er / S, eye)
        ell(d, S, hx + 0.030, hy - 0.012, er * 0.5 / S, er * 0.5 / S, (12, 20, 10, 255))
    return f

def head_beaver(teeth=True, ear_col=None):
    def f(d, S, rng, hx, hy, dark, body):
        ello(d, S, hx - 0.035, hy - 0.062, 0.026, 0.026, dark, ear_col or shade(body, 0.85))
        ello(d, S, hx, hy, 0.080, 0.074, dark, body)
        poly(d, S, [(hx + 0.03, hy + 0.005), (hx + 0.095, hy + 0.022),
                    (hx + 0.03, hy + 0.048)], shade(body, 0.94))
        ell(d, S, hx + 0.092, hy + 0.020, 0.011, 0.009, (26, 20, 16, 255))
        if teeth:  # prominent incisors
            d.rectangle([(hx + 0.052) * S, (hy + 0.048) * S, (hx + 0.066) * S, (hy + 0.075) * S],
                        fill=(250, 248, 240, 255))
            d.rectangle([(hx + 0.068) * S, (hy + 0.048) * S, (hx + 0.080) * S, (hy + 0.072) * S],
                        fill=(240, 236, 224, 255))
        er = max(1.2, S * 0.018)
        ell(d, S, hx + 0.026, hy - 0.014, er / S, er / S, (18, 14, 10, 255))
    return f

def head_rabbit(nose=(232, 150, 160, 255)):
    def f(d, S, rng, hx, hy, dark, body):
        for ex, ln in ((hx - 0.030, 0.20), (hx + 0.012, 0.185)):  # long ears laid up-back
            poly(d, S, [(ex, hy - 0.05), (ex - 0.055, hy - 0.05 - ln),
                        (ex - 0.018, hy - 0.055)], dark)
            poly(d, S, [(ex - 0.004, hy - 0.052), (ex - 0.042, hy - 0.05 - ln * 0.88),
                        (ex - 0.016, hy - 0.056)], shade(body, 0.9))
        ello(d, S, hx, hy, 0.072, 0.066, dark, body)
        ell(d, S, hx + 0.068, hy + 0.018, 0.011, 0.009, nose)
        er = max(1.2, S * 0.019)
        ell(d, S, hx + 0.024, hy - 0.012, er / S, er / S, (20, 14, 12, 255))
    return f

# --- body markings ---
def mark_skunk_stripe(d, S, rng, geo, dark, body):
    bcx, bcy, brx, bry = geo
    poly(d, S, [(bcx - brx * 0.95, bcy - bry * 0.55), (bcx - brx * 0.2, bcy - bry * 0.98),
                (bcx + brx * 0.75, bcy - bry * 0.92), (bcx + brx * 0.95, bcy - bry * 0.45),
                (bcx + brx * 0.4, bcy - bry * 0.62), (bcx - brx * 0.3, bcy - bry * 0.66)],
         (244, 244, 240, 255))

def mark_bristles(d, S, rng, geo, dark, body):  # dambreaker kit
    bcx, bcy, brx, bry = geo
    for i in range(11):
        t = i / 10.0
        x = bcx - brx * 0.85 + t * brx * 1.6
        y = bcy - bry * (0.92 - 0.25 * abs(t - 0.4))
        line(d, S, [(x, y), (x + 0.008, y - 0.045 - 0.02 * rng.random())],
             (52, 38, 26, 255), w=wline(S))

def mark_cord_fur(d, S, rng, geo, dark, body):  # Loper legs: long cord-like fur
    bcx, bcy, brx, bry = geo
    for lx in (bcx - brx * 0.66, bcx + brx * 0.66, bcx - brx * 0.56, bcx + brx * 0.58):
        for k in range(3):
            dx = (k - 1) * 0.016
            pts = [(lx + dx, bcy + bry * 0.6), (lx + dx + 0.006, bcy + bry * 1.6),
                   (lx + dx - 0.004, bcy + bry * 2.6)]
            line(d, S, pts, shade(body, 0.72), w=wline(S))

def mark_scales(d, S, rng, geo, dark, body):  # wyrmhound body scale rows
    bcx, bcy, brx, bry = geo
    for row in range(3):
        for i in range(7):
            x = bcx - brx * 0.78 + i * brx * 0.24 + (row % 2) * brx * 0.12
            y = bcy - bry * 0.55 + row * bry * 0.5
            d.arc([(x - 0.028) * S, (y - 0.02) * S, (x + 0.028) * S, (y + 0.025) * S],
                  start=20, end=160, fill=shade(body, 0.62), width=wline(S))

def mark_bald_stubs(d, S, rng, geo, dark, body):  # plucked-chicken feather stubs + wrinkles
    bcx, bcy, brx, bry = geo
    for i in range(9):
        x = bcx - brx * 0.8 + rng.random() * brx * 1.6
        y = bcy - bry * 0.6 + rng.random() * bry * 0.9
        line(d, S, [(x, y), (x + 0.012, y - 0.02)], (238, 228, 210, 255), w=wline(S))
    for i in range(3):
        y = bcy - bry * 0.2 + i * bry * 0.35
        d.arc([(bcx - brx * 0.5) * S, (y - 0.03) * S, (bcx + brx * 0.3) * S, (y + 0.035) * S],
              start=30, end=150, fill=shade(body, 0.82), width=wline(S))

def head_wyrmhound(d, S, rng, hx, hy, dark, body):
    # serpentine head: long low skull, yellow eye, twin fangs, forked tongue
    poly(d, S, [(hx - 0.05, hy - 0.045), (hx + 0.10, hy - 0.028), (hx + 0.145, hy + 0.008),
                (hx + 0.10, hy + 0.042), (hx - 0.04, hy + 0.05)], dark)
    poly(d, S, [(hx - 0.042, hy - 0.036), (hx + 0.095, hy - 0.020), (hx + 0.132, hy + 0.007),
                (hx + 0.095, hy + 0.034), (hx - 0.032, hy + 0.041)], body)
    for i in range(4):  # scale arcs on the skull
        x = hx - 0.01 + i * 0.032
        d.arc([(x - 0.02) * S, (hy - 0.045) * S, (x + 0.02) * S, (hy - 0.005) * S],
              start=20, end=160, fill=shade(body, 0.62), width=wline(S))
    er = max(1.4, S * 0.023)  # unsettling yellow eye
    ell(d, S, hx + 0.045, hy - 0.014, er / S, er / S, (238, 196, 32, 255))
    ell(d, S, hx + 0.045, hy - 0.014, er * 0.4 / S, er * 0.4 / S, (16, 12, 8, 255))
    for fx in (0.105, 0.125):  # vicious fangs at the front of the muzzle
        poly(d, S, [(hx + fx, hy + 0.030), (hx + fx + 0.012, hy + 0.030),
                    (hx + fx + 0.006, hy + 0.058)], (248, 246, 238, 255))
    line(d, S, [(hx + 0.148, hy + 0.010), (hx + 0.175, hy + 0.014)], (196, 52, 52, 255), w=wline(S))
    line(d, S, [(hx + 0.175, hy + 0.014), (hx + 0.188, hy + 0.004)], (196, 52, 52, 255), w=wline(S))
    line(d, S, [(hx + 0.175, hy + 0.014), (hx + 0.188, hy + 0.024)], (196, 52, 52, 255), w=wline(S))

def draw_mole(S, rng):
    body, dark = (96, 70, 52, 255), (40, 30, 22, 255)
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # cylindrical velvet body, low to the ground
    ello(d, S, 0.44, 0.70, 0.30, 0.115, dark, body)
    ell(d, S, 0.36, 0.665, 0.17, 0.05, shade(body, 1.14))  # velvet sheen
    line(d, S, [(0.16, 0.68), (0.07, 0.64)], dark, w=wline(S) + 1)  # thin tail
    # pointed snout cone at the front, tiny inconspicuous eye
    poly(d, S, [(0.66, 0.635), (0.86, 0.685), (0.66, 0.735)], dark)
    poly(d, S, [(0.67, 0.648), (0.845, 0.685), (0.67, 0.722)], shade(body, 1.05))
    ell(d, S, 0.855, 0.685, 0.012, 0.010, (226, 158, 148, 255))  # nose tip
    ell(d, S, 0.665, 0.662, 0.007, 0.007, (16, 12, 10, 255))
    # huge pale digging forepaws with claws, front and rear
    for px, py in ((0.63, 0.815), (0.30, 0.815)):
        ello(d, S, px, py, 0.052, 0.034, dark, (232, 178, 160, 255))
        for k in range(3):
            cx = px + 0.028 + k * 0.012
            line(d, S, [(cx, py + 0.012), (cx + 0.014, py + 0.030)],
                 (240, 226, 210, 255), w=wline(S))
    return img

def draw_porcupine(S, rng):
    body, dark = (74, 54, 38, 255), (34, 26, 20, 255)
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    for lx in (0.30, 0.58):  # stubby legs under the quill mass
        line(d, S, [(lx, 0.76), (lx, 0.885)], dark, w=wline(S) + 2)
    ello(d, S, 0.44, 0.665, 0.315, 0.155, dark, body)
    # banded quills radiating over the back and rear
    for i in range(52):
        t = rng.random()
        x = 0.16 + t * 0.52
        y = 0.665 - 0.145 * math.sin(math.pi * min(max((x - 0.13) / 0.62, 0.0), 1.0))
        ang = -0.5 - t * 1.5
        ln = 0.085 + rng.random() * 0.055
        x2, y2 = x + math.sin(ang) * ln * 0.6 - 0.02, y - math.cos(ang) * ln
        line(d, S, [(x, y), (x2, y2)], (28, 22, 18, 255), w=wline(S))
        tipx, tipy = x + (x2 - x) * 0.55, y + (y2 - y) * 0.55
        line(d, S, [(tipx, tipy), (x2, y2)], (240, 236, 224, 255), w=wline(S))
    # small head poking out at the front, a few short quills on top
    ello(d, S, 0.765, 0.705, 0.075, 0.064, dark, shade(body, 1.08))
    for k in range(4):
        x = 0.715 + k * 0.024
        line(d, S, [(x, 0.645), (x - 0.008, 0.60)], (240, 236, 224, 255), w=wline(S))
    ell(d, S, 0.815, 0.715, 0.011, 0.009, (22, 16, 14, 255))
    er = max(1.2, S * 0.018)
    ell(d, S, 0.775, 0.685, er / S, er / S, (16, 12, 10, 255))
    return img

def draw_raccoon_mutant(S, rng):
    body, dark = (172, 170, 166, 255), (44, 40, 38, 255)
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # ringed tail curling behind, drawn first
    pts = [(0.30, 0.82), (0.16, 0.78), (0.10, 0.62), (0.14, 0.48)]
    line(d, S, pts, dark, w=wline(S) + 3)
    line(d, S, pts, shade(body, 0.9), w=wline(S) + 1)
    for t in (0.25, 0.55, 0.85):
        i = min(int(t * (len(pts) - 1) * 10), 10 * (len(pts) - 1) - 1)
        seg = pts[i // 10]; nxt = pts[min(i // 10 + 1, len(pts) - 1)]
        x = seg[0] + (nxt[0] - seg[0]) * (i % 10) / 10.0
        y = seg[1] + (nxt[1] - seg[1]) * (i % 10) / 10.0
        ell(d, S, x, y, 0.020, 0.020, dark)
    # hindlimb stance: feet on the ground, body tilted upright
    for fx in (0.40, 0.56):
        line(d, S, [(fx, 0.72), (fx, 0.885)], dark, w=wline(S) + 2)
        line(d, S, [(fx, 0.72), (fx, 0.885)], body, w=wline(S))
        ell(d, S, fx + 0.018, 0.885, 0.032, 0.014, body)
    ello(d, S, 0.48, 0.55, 0.165, 0.235, dark, body)  # upright torso
    ell(d, S, 0.48, 0.63, 0.115, 0.15, shade(body, 1.1))  # pale belly
    # long fingered forepaws raised at chest height
    for ax, ay in ((0.335, 0.52), (0.625, 0.52)):
        line(d, S, [(0.48 + (ax - 0.48) * 0.5, 0.55), (ax, ay)], dark, w=wline(S) + 1)
        ell(d, S, ax, ay, 0.026, 0.020, shade(body, 0.92))
        for k in range(3):  # elongated fingers
            line(d, S, [(ax - 0.012 + k * 0.012, ay - 0.012),
                        (ax - 0.016 + k * 0.014, ay - 0.038)], dark, w=wline(S))
    # head with the raccoon mask
    for ex in (0.415, 0.545):
        poly(d, S, [(ex, 0.315), (ex + 0.006, 0.245), (ex + 0.038, 0.305)], dark)
    ello(d, S, 0.48, 0.36, 0.105, 0.092, dark, body)
    poly(d, S, [(0.385, 0.335), (0.575, 0.335), (0.565, 0.385), (0.395, 0.385)], dark)
    poly(d, S, [(0.398, 0.343), (0.562, 0.343), (0.554, 0.378), (0.406, 0.378)],
         (52, 46, 44, 255))  # mask band
    for ex in (0.435, 0.525):
        er = max(1.2, S * 0.018)
        ell(d, S, ex, 0.358, er / S, er / S, (240, 238, 230, 255))
    poly(d, S, [(0.455, 0.415), (0.505, 0.415), (0.48, 0.445)], shade(body, 0.9))
    ell(d, S, 0.48, 0.418, 0.012, 0.009, (22, 18, 16, 255))
    return img

# ---------------------------------------------------------------- birds
def draw_bird(S, rng, body, dark, *, belly=None, wing=None, head_col=None,
              beak_col=(38, 30, 22, 255), beak_len=0.085, tail_col=None,
              crest=False, mask=False, cap=None, bib=False, helmet=False,
              spots=False, tail_tip=None, wing_dot=None):
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    ground = 0.90
    bcx, bcy, brx, bry = 0.435, 0.575, 0.255, 0.195
    tc = tail_col or shade(body, 0.85)
    poly(d, S, [(bcx - brx * 0.75, bcy + 0.05), (bcx - brx * 1.05, bcy + 0.30),
                (bcx - brx * 0.55, bcy + 0.33), (bcx - brx * 0.30, bcy + 0.12)], dark)
    poly(d, S, [(bcx - brx * 0.72, bcy + 0.07), (bcx - brx * 0.97, bcy + 0.285),
                (bcx - brx * 0.55, bcy + 0.30), (bcx - brx * 0.33, bcy + 0.12)], tc)
    if tail_tip:
        poly(d, S, [(bcx - brx * 1.05, bcy + 0.30), (bcx - brx * 0.55, bcy + 0.33),
                    (bcx - brx * 0.60, bcy + 0.27), (bcx - brx * 0.97, bcy + 0.245)], tail_tip)
    for lx in (0.42, 0.50):  # thin legs + feet
        line(d, S, [(lx, bcy + bry * 0.82), (lx, ground)], (52, 42, 30, 255), w=wline(S))
        line(d, S, [(lx - 0.018, ground), (lx + 0.024, ground)], (52, 42, 30, 255), w=wline(S))
    ello(d, S, bcx, bcy, brx, bry, dark, body)
    if belly:
        ell(d, S, bcx + brx * 0.18, bcy + bry * 0.38, brx * 0.66, bry * 0.55, belly)
    wc = wing or shade(body, 0.82)
    poly(d, S, [(bcx - brx * 0.55, bcy - bry * 0.45), (bcx + brx * 0.45, bcy - bry * 0.30),
                (bcx + brx * 0.30, bcy + bry * 0.30), (bcx - brx * 0.45, bcy + bry * 0.22)], dark)
    poly(d, S, [(bcx - brx * 0.50, bcy - bry * 0.40), (bcx + brx * 0.40, bcy - bry * 0.25),
                (bcx + brx * 0.25, bcy + bry * 0.24), (bcx - brx * 0.40, bcy + bry * 0.17)], wc)
    if spots:  # guineafowl white-spotted plumage
        for i in range(26):
            x = bcx - brx * 0.8 + rng.random() * brx * 1.6
            y = bcy - bry * 0.6 + rng.random() * bry * 1.25
            if ((x - bcx) / brx) ** 2 + ((y - bcy) / bry) ** 2 < 0.82:
                r = 0.008 + rng.random() * 0.008
                ell(d, S, x, y, r, r, (238, 238, 232, 255))
    if wing_dot:
        ell(d, S, bcx + brx * 0.28, bcy - bry * 0.05, 0.016, 0.012, wing_dot)
    hx, hy, hr = 0.665, 0.335, 0.112
    ello(d, S, hx, hy, hr, hr * 0.94, dark, head_col or body)
    if cap:  # black cap over the crown
        d.pieslice([(hx - hr) * S, (hy - hr * 0.94) * S, (hx + hr) * S, (hy + hr * 0.94) * S],
                   start=180, end=360, fill=cap)
        ell(d, S, hx - hr * 0.35, hy + hr * 0.28, hr * 0.52, hr * 0.42, (246, 246, 242, 255))
    if bib:
        ell(d, S, hx + hr * 0.30, hy + hr * 0.52, hr * 0.34, hr * 0.36, cap or (18, 16, 16, 255))
    if mask:  # waxwing black mask through the eye, white-edged
        poly(d, S, [(hx - hr * 0.55, hy - hr * 0.18), (hx + hr * 0.98, hy - hr * 0.30),
                    (hx + hr * 0.90, hy + hr * 0.16), (hx - hr * 0.45, hy + hr * 0.10)],
             (250, 250, 246, 255))
        poly(d, S, [(hx - hr * 0.50, hy - hr * 0.10), (hx + hr * 0.94, hy - hr * 0.21),
                    (hx + hr * 0.86, hy + hr * 0.08), (hx - hr * 0.42, hy + hr * 0.03)],
             (16, 14, 14, 255))
    if crest:  # swept-back crest
        poly(d, S, [(hx - hr * 0.15, hy - hr * 0.80), (hx - hr * 0.95, hy - hr * 1.25),
                    (hx + hr * 0.30, hy - hr * 0.55)], dark)
        poly(d, S, [(hx - hr * 0.10, hy - hr * 0.78), (hx - hr * 0.78, hy - hr * 1.12),
                    (hx + hr * 0.24, hy - hr * 0.56)], head_col or body)
    if helmet:  # guineafowl bony casque + wattles
        ello(d, S, hx - hr * 0.05, hy - hr * 1.02, hr * 0.26, hr * 0.30, dark, (222, 206, 172, 255))
        ell(d, S, hx + hr * 0.58, hy + hr * 0.40, hr * 0.16, hr * 0.24, (200, 58, 46, 255))
    # beak
    poly(d, S, [(hx + hr * 0.72, hy - hr * 0.12), (hx + hr * 0.72 + beak_len, hy + hr * 0.06),
                (hx + hr * 0.72, hy + hr * 0.26)], dark)
    poly(d, S, [(hx + hr * 0.74, hy - hr * 0.06), (hx + hr * 0.70 + beak_len, hy + hr * 0.06),
                (hx + hr * 0.74, hy + hr * 0.20)], beak_col)
    if not mask:
        er = max(1.2, S * 0.020)
        ell(d, S, hx + hr * 0.30, hy - hr * 0.08, er / S, er / S, (16, 12, 10, 255))
    return img

def draw_chick(S, rng, down, dark, *, belly=None, bill="point",
               bill_col=(206, 158, 62, 255), stripes=False, head_patch=None,
               scruffy=False, scale=1.0, neck=0.0):
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    ground = 0.90
    bcx, bcy = 0.465, 0.665
    br = 0.165 * scale
    for lx in (bcx - 0.045, bcx + 0.05):  # stick legs
        line(d, S, [(lx, bcy + br * 0.8), (lx, ground)], (120, 92, 48, 255), w=wline(S))
        line(d, S, [(lx - 0.016, ground), (lx + 0.020, ground)], (120, 92, 48, 255), w=wline(S))
    # downy body: scalloped fluff edge under a filled core
    for i in range(14):
        a = i / 14.0 * 2 * math.pi
        x, y = bcx + math.cos(a) * br * 0.92, bcy + math.sin(a) * br * 0.88
        ell(d, S, x, y, br * 0.30, br * 0.30, dark)
    ell(d, S, bcx, bcy, br, br * 0.94, dark)
    ell(d, S, bcx, bcy, br * 0.88, br * 0.82, down)
    if belly:
        ell(d, S, bcx + br * 0.15, bcy + br * 0.30, br * 0.55, br * 0.45, belly)
    if stripes:  # grebe chick: dark stripes over back and head
        for t in (-0.55, -0.18, 0.18, 0.55):
            x = bcx + t * br
            poly(d, S, [(x - 0.012, bcy - br * 0.80), (x + 0.012, bcy - br * 0.80),
                        (x + 0.016, bcy + br * 0.30), (x - 0.016, bcy + br * 0.30)],
                 shade(down, 0.55))
    hx, hy, hr = bcx + br * 0.78, bcy - br * 0.72 - neck, 0.098 * scale
    ell(d, S, hx, hy, hr * 1.06, hr * 1.0, dark)
    ell(d, S, hx, hy, hr * 0.92, hr * 0.86, down)
    if head_patch:
        ell(d, S, hx, hy - hr * 0.30, hr * 0.62, hr * 0.48, head_patch)
    if stripes:
        for t in (-0.35, 0.30):
            line(d, S, [(hx + t * hr, hy - hr * 0.80), (hx + t * hr, hy + hr * 0.25)],
                 shade(down, 0.55), w=wline(S) + 1)
    if scruffy:  # pigeon squab: sparse long down strands
        for i in range(10):
            a = rng.random() * 2 * math.pi
            x, y = bcx + math.cos(a) * br * 0.8, bcy + math.sin(a) * br * 0.7
            line(d, S, [(x, y), (x + math.cos(a) * 0.035, y + math.sin(a) * 0.035 - 0.012)],
                 shade(down, 1.18), w=wline(S))
    if bill == "flat":  # duckling
        ello(d, S, hx + hr * 1.02, hy + hr * 0.16, hr * 0.52, hr * 0.24, dark, bill_col)
    elif bill == "long":
        poly(d, S, [(hx + hr * 0.6, hy - hr * 0.05), (hx + hr * 1.85, hy + hr * 0.16),
                    (hx + hr * 0.6, hr * 0.30 + hy)], dark)
        poly(d, S, [(hx + hr * 0.62, hy), (hx + hr * 1.78, hy + hr * 0.15),
                    (hx + hr * 0.62, hy + hr * 0.24)], bill_col)
    else:
        poly(d, S, [(hx + hr * 0.62, hy - hr * 0.10), (hx + hr * 1.30, hy + hr * 0.10),
                    (hx + hr * 0.62, hr * 0.28 + hy)], dark)
        poly(d, S, [(hx + hr * 0.64, hy - hr * 0.04), (hx + hr * 1.22, hy + hr * 0.10),
                    (hx + hr * 0.64, hy + hr * 0.22)], bill_col)
    er = max(1.1, S * 0.017)
    ell(d, S, hx + hr * 0.25, hy - hr * 0.10, er / S, er / S, (16, 12, 10, 255))
    return img

# ---------------------------------------------------------------- corpses
def draw_prone(S, rng, skin, cloth, dark, *, scale=1.0, charred=False):
    """Placeholder corpse: a plain prone figure, muted tones, no gore."""
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    gy = 0.815  # ground line the body rests on
    x0, x1 = 0.5 - 0.37 * scale, 0.5 + 0.27 * scale
    th = 0.105 * scale  # torso thickness
    # feet at the far end
    ell(d, S, x0 - 0.012, gy - 0.018, 0.020, 0.030 * scale + 0.012, dark)
    ell(d, S, x0 - 0.012, gy - 0.018, 0.013, 0.022 * scale + 0.008, skin)
    # legs + torso as one prone mass, shoulders slightly raised
    poly(d, S, [(x0, gy - th * 0.72), (x1 - 0.06 * scale, gy - th * 0.80),
                (x1, gy - th * 1.02), (x1 + 0.025, gy - th * 0.30),
                (x1 + 0.01, gy), (x0 - 0.005, gy)], dark)
    poly(d, S, [(x0 + 0.010, gy - th * 0.62), (x1 - 0.055 * scale, gy - th * 0.70),
                (x1 - 0.004, gy - th * 0.90), (x1 + 0.012, gy - th * 0.30),
                (x1, gy - 0.008), (x0 + 0.004, gy - 0.008)], cloth)
    line(d, S, [(x0 + 0.045, gy - 0.016), (x1 - 0.17 * scale, gy - 0.016)],
         shade(cloth, 0.78), w=wline(S))  # leg separation seam
    ell(d, S, x1 - 0.10 * scale, gy - th * 0.52, 0.055 * scale, th * 0.34,
        shade(cloth, 1.10))  # shoulder highlight
    # one arm resting alongside the torso, hand past the hip
    line(d, S, [(x1 - 0.05 * scale, gy - th * 0.55), (x1 - 0.24 * scale, gy - 0.012)],
         dark, w=wline(S) + 2)
    line(d, S, [(x1 - 0.05 * scale, gy - th * 0.55), (x1 - 0.24 * scale, gy - 0.012)],
         skin, w=wline(S))
    # head, turned to the side, eyes closed
    hx, hy, hr = x1 + 0.085 * scale, gy - 0.045 * scale, 0.068 * scale
    ello(d, S, hx, hy, hr, hr * 0.90, dark, skin)
    line(d, S, [(hx + hr * 0.22, hy + hr * 0.02), (hx + hr * 0.60, hy + hr * 0.02)],
         shade(skin, 0.55), w=wline(S))  # closed eye line
    ell(d, S, hx - hr * 0.30, hy - hr * 0.42, hr * 0.55, hr * 0.30,
        shade(skin, 0.82))  # hair mass
    if charred:  # char cracks in burnt-brown over the blackened mass
        for i in range(10):
            x = x0 + 0.03 + rng.random() * (x1 - x0 - 0.05)
            y = gy - th * 0.75 + rng.random() * th * 0.6
            line(d, S, [(x, y), (x + 0.018 + rng.random() * 0.02, y + 0.006)],
                 (122, 74, 42, 255), w=wline(S))
    return img

# --- extra species heads / tails / marks ---
def head_ferret(d, S, rng, hx, hy, dark, body):
    ello(d, S, hx - 0.032, hy - 0.055, 0.024, 0.024, dark, shade(body, 0.9))  # round ear
    ello(d, S, hx, hy, 0.074, 0.068, dark, body)
    poly(d, S, [(hx + 0.02, hy - 0.01), (hx + 0.088, hy + 0.014),
                (hx + 0.02, hy + 0.042)], shade(body, 0.95))
    # dark sable mask across the eyes
    poly(d, S, [(hx - 0.045, hy - 0.028), (hx + 0.075, hy - 0.020),
                (hx + 0.070, hy + 0.016), (hx - 0.040, hy + 0.010)], (74, 58, 44, 255))
    er = max(1.1, S * 0.017)
    ell(d, S, hx + 0.028, hy - 0.006, er / S, er / S, (240, 236, 226, 255))
    ell(d, S, hx + 0.086, hy + 0.013, 0.010, 0.008, (216, 130, 130, 255))  # pink nose

def head_shrew(d, S, rng, hx, hy, dark, body):
    ello(d, S, hx - 0.028, hy - 0.048, 0.020, 0.020, dark, shade(body, 0.9))
    ello(d, S, hx, hy, 0.062, 0.056, dark, body)
    poly(d, S, [(hx + 0.02, hy - 0.012), (hx + 0.125, hy + 0.016),
                (hx + 0.02, hy + 0.036)], dark)  # very long pointed snout
    poly(d, S, [(hx + 0.022, hy - 0.006), (hx + 0.112, hy + 0.015),
                (hx + 0.022, hy + 0.029)], shade(body, 1.02))
    ell(d, S, hx + 0.118, hy + 0.015, 0.008, 0.007, (216, 150, 140, 255))
    er = max(1.0, S * 0.014)
    ell(d, S, hx + 0.018, hy - 0.010, er / S, er / S, (16, 12, 10, 255))

def head_skunk(d, S, rng, hx, hy, dark, body):
    poly(d, S, [(hx - 0.042, hy - 0.04), (hx - 0.046, hy - 0.105),
                (hx + 0.002, hy - 0.062)], dark)  # small point ear
    ello(d, S, hx, hy, 0.074, 0.068, dark, body)
    poly(d, S, [(hx + 0.025, hy + 0.008), (hx + 0.082, hy + 0.026),
                (hx + 0.025, hy + 0.042)], shade(body, 1.25))
    # white forehead stripe running up from the nose
    poly(d, S, [(hx + 0.055, hy + 0.018), (hx + 0.030, hy - 0.030),
                (hx + 0.006, hy - 0.062), (hx + 0.034, hy + 0.010)], (244, 244, 240, 255))
    ell(d, S, hx + 0.080, hy + 0.025, 0.010, 0.008, (24, 20, 20, 255))
    er = max(1.1, S * 0.017)
    ell(d, S, hx + 0.024, hy - 0.008, er / S, er / S, (230, 228, 224, 255))

def tail_skunk(d, S, rng, x, y, dark, body):
    poly(d, S, [(x + 0.03, y + 0.05), (x - 0.115, y - 0.13), (x - 0.155, y - 0.03),
                (x - 0.10, y + 0.11), (x + 0.03, y + 0.12)], dark)
    poly(d, S, [(x + 0.015, y + 0.045), (x - 0.095, y - 0.10), (x - 0.125, y - 0.025),
                (x - 0.08, y + 0.09), (x + 0.015, y + 0.10)], body)
    poly(d, S, [(x - 0.01, y + 0.02), (x - 0.075, y - 0.075), (x - 0.095, y - 0.02),
                (x - 0.05, y + 0.06)], (244, 244, 240, 255))  # white plume core

def mark_thrasher_claws(big=True):
    def f(d, S, rng, geo, dark, body):
        bcx, bcy, brx, bry = geo
        lx = bcx + brx * 0.66
        n = 3 if big else 2
        for k in range(n):  # long digging nails raking forward from the paw
            x = lx + 0.018 + k * 0.016
            line(d, S, [(x, 0.878), (x + 0.030, 0.898)], (238, 230, 214, 255), w=wline(S))
    return f

# ---------------------------------------------------------------- jobs
DARK = (36, 29, 22, 255)

def J(mid, fn):
    return (f"{SHEET}/{mid}/{mid}", 32, fn)

JOBS = [
    # ---- birds: adults ----
    J("mon_chickadee", lambda S, r: draw_bird(
        S, r, (146, 144, 138, 255), DARK, belly=(233, 223, 197, 255),
        wing=(94, 92, 88, 255), head_col=(246, 246, 242, 255),
        cap=(18, 16, 16, 255), bib=True, beak_col=(30, 28, 26, 255), beak_len=0.045)),
    J("mon_waxwing", lambda S, r: draw_bird(
        S, r, (178, 158, 128, 255), DARK, belly=(228, 208, 142, 255),
        wing=(150, 130, 102, 255), head_col=(190, 168, 136, 255),
        crest=True, mask=True, tail_tip=(242, 216, 94, 255),
        wing_dot=(208, 62, 46, 255), beak_col=(30, 26, 22, 255), beak_len=0.045)),
    J("mon_guinea_fowl", lambda S, r: draw_bird(
        S, r, (128, 128, 134, 255), DARK, belly=(150, 150, 156, 255),
        wing=(102, 102, 110, 255), head_col=(172, 164, 148, 255),
        helmet=True, spots=True, beak_col=(90, 78, 62, 255), beak_len=0.05)),
    # ---- birds: chicks (palettes resolve through the generic chick abstracts) ----
    J("mon_chickadee_chick", lambda S, r: draw_chick(S, r, (168, 164, 156, 255), DARK)),
    J("mon_hummingbird_chick", lambda S, r: draw_chick(S, r, (150, 146, 140, 255), DARK, scale=0.8)),
    J("mon_waxwing_chick", lambda S, r: draw_chick(S, r, (164, 158, 148, 255), DARK)),
    J("mon_woodpecker_chick", lambda S, r: draw_chick(S, r, (174, 160, 150, 255), DARK)),
    J("mon_coot_chick", lambda S, r: draw_chick(
        S, r, (54, 50, 50, 255), DARK, head_patch=(218, 112, 52, 255))),
    J("mon_cormorant_chick", lambda S, r: draw_chick(
        S, r, (72, 64, 58, 255), DARK, bill="long", neck=0.03)),
    J("mon_duck_domestic_chick", lambda S, r: draw_chick(
        S, r, (246, 214, 92, 255), DARK, bill="flat", bill_col=(234, 166, 86, 255))),
    J("mon_goose_chick", lambda S, r: draw_chick(
        S, r, (230, 208, 122, 255), DARK, bill="flat", bill_col=(122, 112, 92, 255), scale=1.1)),
    J("mon_grebe_chick", lambda S, r: draw_chick(
        S, r, (240, 238, 232, 255), DARK, stripes=True)),
    J("mon_moorhen_chick", lambda S, r: draw_chick(
        S, r, (50, 46, 46, 255), DARK, bill_col=(208, 72, 46, 255))),
    J("mon_pigeon_chick", lambda S, r: draw_chick(
        S, r, (208, 180, 112, 255), DARK, scruffy=True, bill_col=(72, 60, 50, 255))),
    J("mon_turkey_domestic_chick", lambda S, r: draw_chick(
        S, r, (170, 140, 98, 255), DARK, belly=(208, 184, 142, 255))),
    J("mon_guinea_fowl_chick", lambda S, r: draw_chick(
        S, r, (244, 208, 88, 255), DARK, belly=(216, 180, 112, 255))),
    # ---- mammals ----
    J("mon_beaver_kit", lambda S, r: quadruped(
        S, r, (122, 84, 52, 255), DARK, bulk=0.85, leg_h=0.22,
        belly=(152, 118, 82, 255), tail=tail_flat_beaver, head=head_beaver())),
    J("mon_cat_black", lambda S, r: quadruped(
        S, r, (30, 28, 30, 255), (10, 10, 12, 255), bulk=0.8, leg_h=0.27,
        tail=tail_cat_up, head=head_cat())),
    J("mon_cat_black_kitten", lambda S, r: quadruped(
        S, r, (30, 28, 30, 255), (10, 10, 12, 255), bulk=0.55, leg_h=0.20,
        tail=tail_cat_up, head=head_cat(kitten=True))),
    J("mon_cat_isherwood", lambda S, r: quadruped(
        S, r, (168, 168, 164, 255), DARK, bulk=0.8, leg_h=0.27,
        belly=(196, 196, 190, 255), tail=tail_cat_up, head=head_cat(eye=(208, 162, 62, 255)))),
    J("mon_coyote_pup", lambda S, r: quadruped(
        S, r, (164, 130, 88, 255), DARK, bulk=0.7, leg_h=0.23,
        belly=(208, 184, 146, 255), tail=tail_dog_wag, head=head_dog(snout=0.09))),
    J("mon_ferret", lambda S, r: quadruped(
        S, r, (216, 204, 178, 255), DARK, bulk=0.62, leg_h=0.16, long_body=1.25,
        tail=tail_long_low, head=head_ferret, leg_col=(74, 60, 46, 255))),
    J("mon_ferret_kit", lambda S, r: quadruped(
        S, r, (228, 218, 196, 255), DARK, bulk=0.40, leg_h=0.10, long_body=1.2,
        tail=tail_long_low, head=head_ferret, leg_col=(120, 104, 84, 255))),
    J("mon_hare_baby", lambda S, r: quadruped(
        S, r, (150, 128, 100, 255), DARK, bulk=0.50, leg_h=0.16,
        belly=(228, 220, 202, 255), tail=tail_cotton, head=head_rabbit())),
    J("mon_rabbit_baby", lambda S, r: quadruped(
        S, r, (198, 188, 174, 255), (118, 110, 98, 255), bulk=0.48, leg_h=0.15,
        belly=(238, 232, 222, 255), tail=tail_cotton, head=head_rabbit())),
    J("mon_wild_rabbit_baby", lambda S, r: quadruped(
        S, r, (142, 116, 84, 255), DARK, bulk=0.48, leg_h=0.15,
        belly=(216, 202, 178, 255), tail=tail_cotton, head=head_rabbit())),
    J("mon_mole", draw_mole),
    J("mon_porcupine", draw_porcupine),
    J("mon_shrew", lambda S, r: quadruped(
        S, r, (128, 102, 74, 255), DARK, bulk=0.42, leg_h=0.12,
        tail=tail_thin(), head=head_shrew)),
    J("mon_skunk", lambda S, r: quadruped(
        S, r, (28, 26, 28, 255), (8, 8, 10, 255), bulk=0.78, leg_h=0.20,
        tail=tail_skunk, head=head_skunk, mark=mark_skunk_stripe)),
    J("mon_human", lambda S, r: draw_prone(
        S, r, (146, 164, 124, 255), (96, 96, 104, 255), DARK)),
    J("mon_child", lambda S, r: draw_prone(
        S, r, (146, 164, 124, 255), (106, 102, 98, 255), DARK, scale=0.74)),
    J("mon_human_scorched", lambda S, r: draw_prone(
        S, r, (56, 42, 34, 255), (32, 28, 26, 255), (12, 10, 10, 255), charred=True)),
    # ---- mutant mammals ----
    J("mon_beaver_mutant_avian_kit", lambda S, r: quadruped(
        S, r, (228, 178, 160, 255), (120, 84, 72, 255), bulk=0.85, leg_h=0.22,
        tail=tail_flat_beaver, head=head_beaver(ear_col=(228, 178, 160, 255)),
        mark=mark_bald_stubs, leg_col=(228, 178, 160, 255))),
    J("mon_beaver_mutant_huge_kit", lambda S, r: quadruped(
        S, r, (122, 84, 52, 255), DARK, bulk=1.05, leg_h=0.24,
        belly=(152, 118, 82, 255), tail=tail_flat_beaver, head=head_beaver(),
        mark=mark_bristles)),
    J("mon_coyote_pup_mutant_shark", lambda S, r: quadruped(
        S, r, (164, 130, 88, 255), DARK, bulk=0.7, leg_h=0.23,
        belly=(208, 184, 146, 255), tail=tail_dog_wag,
        head=head_dog(snout=0.095, grin=True))),
    J("mon_coyote_pup_mutant_venom", lambda S, r: quadruped(
        S, r, (164, 130, 88, 255), DARK, bulk=0.7, leg_h=0.23,
        belly=(208, 184, 146, 255), tail=tail_dog_wag,
        head=head_dog(snout=0.09, drool=True))),
    J("mon_dog_mutant_loper", lambda S, r: quadruped(
        S, r, (172, 170, 166, 255), DARK, bulk=1.0, leg_h=0.46,
        belly=(198, 196, 190, 255), tail=tail_dog_wag,
        head=head_dog(ear="flop", eye=(228, 234, 240, 255)), mark=mark_cord_fur)),
    J("mon_dog_mutant_loper_pup", lambda S, r: quadruped(
        S, r, (178, 176, 172, 255), DARK, bulk=0.55, leg_h=0.30,
        tail=tail_dog_wag, head=head_dog(ear="flop", snout=0.08, eye=(228, 234, 240, 255)))),
    J("mon_dog_mutant_mauler", lambda S, r: quadruped(
        S, r, (170, 168, 164, 255), DARK, bulk=1.18, leg_h=0.26,
        belly=(198, 196, 190, 255), tail=tail_dog_wag,
        head=head_dog(ear="flop", snout=0.10, jaw=True, teeth=True))),
    J("mon_dog_mutant_mauler_pup", lambda S, r: quadruped(
        S, r, (176, 174, 170, 255), DARK, bulk=0.55, leg_h=0.18,
        tail=tail_dog_wag, head=head_dog(ear="flop", snout=0.07, jaw=True, teeth=True))),
    J("mon_dog_mutant_thrasher", lambda S, r: quadruped(
        S, r, (180, 166, 150, 255), DARK, bulk=1.05, leg_h=0.24, paw_r=0.05,
        belly=(202, 190, 172, 255), tail=tail_dog_wag,
        head=head_dog(ear="flop", snout=0.10), mark=mark_thrasher_claws(True))),
    J("mon_dog_mutant_thrasher_pup", lambda S, r: quadruped(
        S, r, (184, 172, 158, 255), DARK, bulk=0.55, leg_h=0.18, paw_r=0.045,
        tail=tail_dog_wag, head=head_dog(ear="flop", snout=0.08),
        mark=mark_thrasher_claws(False))),
    J("mon_dog_mutant_wyrmhound", lambda S, r: quadruped(
        S, r, (128, 134, 112, 255), (52, 56, 44, 255), bulk=1.0, leg_h=0.30,
        tail=tail_thin((52, 56, 44, 255)), head=head_wyrmhound, mark=mark_scales)),
    J("mon_dog_mutant_wyrmhound_pup", lambda S, r: quadruped(
        S, r, (136, 142, 120, 255), (52, 56, 44, 255), bulk=0.55, leg_h=0.18,
        tail=tail_thin((52, 56, 44, 255)), head=head_wyrmhound, mark=mark_scales)),
    J("mon_raccoon_mutant", draw_raccoon_mutant),
]

def main():
    written = []
    for rel, size, fn in JOBS:
        mid = os.path.basename(os.path.dirname(rel))
        img = fn(size, rng_for(mid))
        folder = os.path.join(BASE, os.path.dirname(rel))
        os.makedirs(folder, exist_ok=True)
        img.save(os.path.join(BASE, rel + ".png"))
        with open(os.path.join(BASE, rel + ".json"), "w") as fh:
            json.dump([{"id": mid, "fg": mid, "bg": ""}], fh, indent=2)
            fh.write("\n")
        with open(os.path.join(folder, "credits.txt"), "w") as fh:
            fh.write(CREDITS)
        written.append((mid, rel))
        print("wrote", rel + ".png", img.size)
    # contact-sheet preview: uniform 192px cells (32x6)
    cell, cols = 192, 8
    rows = (len(written) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * cell, rows * cell), (56, 60, 64, 255))
    for i, (mid, rel) in enumerate(sorted(written)):
        im = Image.open(os.path.join(BASE, rel + ".png")).convert("RGBA")
        k = cell // im.size[0]
        im = im.resize((im.size[0] * k, im.size[1] * k), Image.NEAREST)
        x = (i % cols) * cell + (cell - im.size[0]) // 2
        y = (i // cols) * cell + (cell - im.size[1]) // 2
        sheet.alpha_composite(im, (x, y))
    out = os.path.join(BASE, "preview_mammals_birds.png")
    sheet.convert("RGB").save(out)
    print("wrote", out, sheet.size)

if __name__ == "__main__":
    main()
