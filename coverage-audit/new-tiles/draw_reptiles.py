#!/usr/bin/env python3
"""
Draw the missing reptile & amphibian sprites from their game descriptions.

Sources (fork-src/cdda/data/json/monsters/reptile_amphibian.json, CDDA
6a3e59e0): 56 monster ids present in the coverage audit's missing_shipped
(category "monster") with no folder yet -- snakes, two skinks, two turtle
hatchlings, tadpoles, frogs/toads and their giant/mega mutant forms.

Sheet placement is by resolved volume (copy-from chain): <100 L -> 32x32,
100-400 L -> 64x64, >400 L -> 96x96. Above 32x32:
  64x64: mon_foul_giganfrog (240 L), mon_foul_megafrog (325 L),
         mon_odd_giganfrog (240 L), mon_pattern_giganfrog (200 L),
         mon_strange_giganfrog (240 L)
  96x96: mon_odd_megafrog (650 L), mon_rattlesnake_mega (620 L),
         mon_strange_megafrog (650 L)

Design notes, per group (all from the entries' own descriptions):
  Snakes: slithering S-curve in profile. Garter snakes get their three
  yellow longitudinal stripes; the worm snake its glossy brown back and
  pink belly; timber rattlers their dark chevron blotches (snakelets have
  only a tail button, the adult mutant a segmented rattle). The
  deathrattle serpent (mon_rattlesnake_mega) gets the club-like tail with
  bone segments rattling in it, skinless red muscle ropes flanking the
  maw, multiple hollow fangs, and shed scale flakes.
  Skinks: five-lined skink -- five pale lines on a dark brown body; the
  hatchling is near-black with bright lines and the juvenile's blue tail.
  Turtles: painted hatchling -- dark olive shell, red marginal marks,
  yellow head stripes; snapping hatchling -- rough jagged shell, long
  sawtooth tail, hooked beak.
  Tadpoles: rotund body + tail in profile, tail fin as a translucent
  overlay (alpha-composited, never a semi-transparent solid body). Each
  species' speckle/belly/tail traits follow its description (the gray
  tadpole's red-orange tail tip with black spots, the shift bigtad's
  crimson-to-yellow gradient fin, the vocal bigtad's near-black tint...).
  Frogs/toads: squat front pose, UltiCa style -- folded hind legs, eye
  bumps, wide mouth. Leopard spots with pale halos, the peeper's dark X,
  the wood frog's eye mask, the palette frog's bright squarish patches,
  the pattern frogs' chaotic static markings and retained gills, the
  many-legged frogs' extra malformed/spindly legs, the hogtoad's tusks,
  the great beaked toad's bony beak and eye rings, glow spots on the
  taser tongue, the vocal bigfrog's inflated throat sac.
Output is UltiCa decomposed layout (png + tile-entry json per folder).
"""
from PIL import Image, ImageDraw, ImageFilter, ImageChops
import json, math, os, random, zlib

BASE = os.path.dirname(os.path.abspath(__file__))
CREDITS = ("Drawn from the monster's in-game description for the CDDA Voxel\n"
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.\n")

def rng_for(mid):
    return random.Random(zlib.crc32(mid.encode()))

def shade(c, f):
    return tuple(max(0, min(255, int(v * f))) for v in c[:3]) + (255,)

def rellipse(cx, cy, rx, ry, ang=0.0, n=24):
    pts = []
    ca, sa = math.cos(ang), math.sin(ang)
    for i in range(n):
        t = i / n * 2 * math.pi
        x, y = rx * math.cos(t), ry * math.sin(t)
        pts.append((cx + x * ca - y * sa, cy + x * sa + y * ca))
    return pts

def poly(d, pts, S, **kw):
    d.polygon([(x * S, y * S) for x, y in pts], **kw)

def ell(d, cx, cy, rx, ry, S, **kw):
    d.ellipse([(cx - rx) * S, (cy - ry) * S, (cx + rx) * S, (cy + ry) * S], **kw)

def clipped(base, layer):
    """Composite `layer` onto `base`, clipped to base's existing alpha."""
    mask = base.split()[3]
    la = ImageChops.multiply(layer.split()[3], mask)
    layer = layer.copy()
    layer.putalpha(la)
    base.alpha_composite(layer)

def speckle(img, rng, colors, n, rmin, rmax, region=None):
    """Opaque speckles clipped to the image's current silhouette."""
    S = img.size[0]
    layer = Image.new("RGBA", img.size, (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    for _ in range(n):
        x = rng.random(); y = rng.random()
        if region and not (region[0] <= x <= region[2] and region[1] <= y <= region[3]):
            continue
        r = (rmin + rng.random() * (rmax - rmin)) * S
        c = colors[rng.randrange(len(colors))]
        d.ellipse([x * S - r, y * S - r, x * S + r, y * S + r], fill=c)
    clipped(img, layer)

# ---------------------------------------------------------------- snakes
def draw_snake(S, rng, base, dark, belly=None, pattern="plain", pat=None,
               rattle=0, club=False, mega=False, thick=0.066, span=0.86):
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    n = 48
    pts = []
    for i in range(n + 1):
        t = i / n
        x = 0.5 + (t - 0.5) * span
        y = 0.52 + 0.105 * math.sin(t * math.pi * 1.7 + 0.4)
        r = thick * (0.30 + 0.70 * math.sin(math.pi * min(0.15 + 0.85 * t, 1.0)))
        if club and t < 0.10:
            r = thick * 0.62  # blunt club tail, no taper
        pts.append((x, y, r))
    outline = shade(base, 0.45)
    # fill-only render for the pattern mask
    fill = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    fd = ImageDraw.Draw(fill)
    for x, y, r in pts:
        fd.ellipse([(x - r) * S, (y - r) * S, (x + r) * S, (y + r) * S], fill=base)
    d = ImageDraw.Draw(img)
    ow = max(1.5, S * 0.012)
    for x, y, r in pts:  # outline pass
        rr = r + ow / S
        d.ellipse([(x - rr) * S, (y - rr) * S, (x + rr) * S, (y + rr) * S], fill=outline)
    img.alpha_composite(fill)
    d = ImageDraw.Draw(img)
    if belly:  # pale belly line along the lower edge of the coil
        bl = [(x, y + r * 0.52, r) for x, y, r in pts]
        d.line([(x * S, y * S) for x, y, _ in bl], fill=belly, width=max(1, int(S * thick * 0.26)))
    if pattern == "stripes":  # garter: longitudinal stripes follow the curve
        for frac in (-0.45, 0.0, 0.45):
            line = [(x, y + frac * r) for x, y, r in pts]
            d.line([(x * S, y * S) for x, y in line], fill=pat, width=max(1, int(S * thick * 0.15)))
    elif pattern == "bands":  # rattler: dark chevron blotches across the back
        layer = Image.new("RGBA", (S, S), (0, 0, 0, 0))
        ld = ImageDraw.Draw(layer)
        for k in range(6):
            t = 0.16 + k * 0.125
            i = int(t * n)
            x, y, r = pts[i]
            chev = [(x - r * 0.9, y + r * 0.55), (x, y - r * 0.95), (x + r * 0.9, y + r * 0.55),
                    (x + r * 0.45, y + r * 0.30), (x, y - r * 0.35), (x - r * 0.45, y + r * 0.30)]
            ld.polygon([(px * S, py * S) for px, py in chev], fill=pat)
        clipped(img, layer)
        d = ImageDraw.Draw(img)
    elif pattern == "speckle":
        speckle(img, rng, [pat], int(S * 0.9), 0.008, 0.02)
        d = ImageDraw.Draw(img)
    # head at the t=1 end of the path
    hx, hy, hr = pts[-1]
    tx, ty, _ = pts[-4]
    ang = math.atan2(hy - ty, hx - tx)
    head_r = thick * (1.25 if mega else 1.05)
    hcx = hx + math.cos(ang) * head_r * 0.7
    hcy = hy + math.sin(ang) * head_r * 0.7
    mgn = head_r * 1.45  # keep the whole head on-canvas
    hcx = min(max(hcx, mgn), 1.0 - mgn)
    hcy = min(max(hcy, head_r * 1.05), 1.0 - head_r * 1.05)
    hp = rellipse(hcx, hcy, head_r * 1.25, head_r * 0.82, ang)
    poly(d, hp, S, fill=outline)
    inner = rellipse(hcx, hcy, head_r * 1.08, head_r * 0.68, ang)
    poly(d, inner, S, fill=base)
    if mega:  # skinless muscle ropes flanking the maw + multiple fangs
        for sgn in (-1, 1):
            mx = hcx + math.cos(ang + sgn * 0.85) * head_r * 0.95
            my = hcy + math.sin(ang + sgn * 0.85) * head_r * 0.95
            d.line([hcx * S, hcy * S, mx * S, my * S], fill=(168, 70, 58, 255),
                   width=max(2, int(S * 0.03)))
            ell(d, mx, my, 0.028, 0.028, S, fill=(188, 88, 70, 255))
        for k in range(4):  # hollow fangs along the jaw
            fa = ang + (k - 1.5) * 0.22
            fx = hcx + math.cos(fa) * head_r * 1.55
            fy = hcy + math.sin(fa) * head_r * 1.55
            tipx = fx + math.cos(fa) * head_r * 0.42
            tipy = fy + math.sin(fa) * head_r * 0.42
            px_, py_ = math.cos(fa + math.pi / 2), math.sin(fa + math.pi / 2)
            fw = head_r * 0.16
            d.polygon([(fx * S + px_ * fw * S, fy * S + py_ * fw * S),
                       (fx * S - px_ * fw * S, fy * S - py_ * fw * S),
                       (tipx * S, tipy * S)], fill=(244, 240, 226, 255))
    # eyes
    for sgn in (-1, 1):
        ex = hcx + math.cos(ang + sgn * 0.7) * head_r * 0.55
        ey = hcy + math.sin(ang + sgn * 0.7) * head_r * 0.55
        er = max(1.2, S * 0.011)
        d.ellipse([ex * S - er, ey * S - er, ex * S + er, ey * S + er], fill=(20, 14, 8, 255))
    # forked tongue
    t0x, t0y = hcx + math.cos(ang) * head_r * 1.9, hcy + math.sin(ang) * head_r * 1.9
    for sgn in (-1, 1):
        ta = ang + sgn * 0.30
        d.line([t0x * S, t0y * S, (t0x + math.cos(ta) * 0.045) * S, (t0y + math.sin(ta) * 0.045) * S],
               fill=(196, 44, 44, 255), width=max(1, S // 32))
    # tail furniture
    ex0, ey0, _ = pts[0]
    ex1, ey1, _ = pts[3]
    tang = math.atan2(ey0 - ey1, ex0 - ex1)
    if club:  # bone segments rattling inside the club tail + loose shards
        for k in range(3):
            bx = ex0 + math.cos(tang) * k * 0.035 - math.cos(tang) * 0.01
            by = ey0 + math.sin(tang) * k * 0.035
            ell(d, bx, by, 0.020, 0.026, S, fill=(226, 218, 198, 255))
            ell(d, bx, by, 0.011, 0.015, S, fill=(198, 188, 164, 255))
        for _ in range(5):
            sx = ex0 + (rng.random() - 0.3) * 0.16
            sy = ey0 + 0.08 + rng.random() * 0.10
            ell(d, sx, sy, 0.012, 0.006, S, fill=(216, 206, 178, 255))
        # shed scale flakes along the body: small crescents under the coil
        for _ in range(5):
            i = rng.randrange(6, n - 4)
            x, y, r = pts[i]
            d.arc([(x - r * 0.34) * S, (y + r * 0.55) * S, (x + r * 0.34) * S,
                   (y + r * 1.1) * S], start=200, end=330, fill=(214, 204, 174, 255),
                  width=max(1, S // 48))
    elif rattle:  # segmented rattle: tan rings shrinking to a button
        for k in range(rattle):
            bx = ex0 + math.cos(tang) * k * 0.030
            by = ey0 + math.sin(tang) * k * 0.030
            rr = 0.020 * (1.0 - k * 0.13)
            ell(d, bx, by, rr * 0.8, rr, S, fill=(206, 182, 128, 255))
            ell(d, bx, by, rr * 0.45, rr * 0.6, S, fill=(168, 144, 92, 255))
    return img

# ---------------------------------------------------------------- skinks
def draw_skink(S, rng, base, lines, tail_col=None, dark=False):
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    outline = shade(base, 0.45)
    cy = 0.54
    # tail: long taper to the left, slight downward curve
    tail = [(0.335, cy - 0.045), (0.20, cy - 0.015), (0.055, cy + 0.075),
            (0.05, cy + 0.105), (0.20, cy + 0.075), (0.335, cy + 0.055)]
    poly(d, tail, S, fill=outline)
    inner_tail = [(0.335, cy - 0.030), (0.20, cy - 0.002), (0.075, cy + 0.078),
                  (0.20, cy + 0.062), (0.335, cy + 0.040)]
    poly(d, inner_tail, S, fill=tail_col or base)
    # far legs
    for lx in (0.40, 0.63):
        d.line([(lx + 0.02) * S, (cy + 0.10) * S, (lx - 0.01) * S, (cy + 0.205) * S],
               fill=outline, width=max(2, S // 14))
        ell(d, lx - 0.015, cy + 0.215, 0.028, 0.012, S, fill=outline)
    # body + head
    ell(d, 0.52, cy, 0.205, 0.148, S, fill=outline)
    ell(d, 0.52, cy, 0.19, 0.135, S, fill=base)
    hp = [(0.685, cy - 0.055), (0.79, cy - 0.040), (0.855, cy + 0.004),
          (0.79, cy + 0.050), (0.685, cy + 0.057)]
    poly(d, hp, S, fill=outline)
    hp2 = [(0.69, cy - 0.042), (0.785, cy - 0.028), (0.832, cy + 0.004),
           (0.785, cy + 0.038), (0.69, cy + 0.045)]
    poly(d, hp2, S, fill=base)
    # the five lines, clipped to the body ellipse so they read as stripes
    # on a dark back instead of blanketing it; short segments on the head
    lmask = Image.new("L", (S, S), 0)
    lmd = ImageDraw.Draw(lmask)
    lmd.ellipse([(0.52 - 0.19) * S, (cy - 0.135) * S, (0.52 + 0.19) * S, (cy + 0.135) * S],
                fill=255)
    llay = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    lld = ImageDraw.Draw(llay)
    for k, frac in enumerate((-0.64, -0.32, 0.0, 0.32, 0.64)):
        yy = cy + frac * 0.135
        lld.line([0.335 * S, yy * S, 0.705 * S, yy * S], fill=lines,
                 width=max(1, int(S * 0.016)))
    la = ImageChops.multiply(llay.split()[3], lmask)
    llay.putalpha(la)
    img.alpha_composite(llay)
    d = ImageDraw.Draw(img)
    for frac in (-0.32, 0.0, 0.32):
        yy = cy + frac * 0.10
        d.line([0.70 * S, yy * S, 0.83 * S, yy * S], fill=lines, width=max(1, int(S * 0.016)))
    # near legs
    for lx in (0.42, 0.65):
        d.line([(lx) * S, (cy + 0.10) * S, (lx + 0.035) * S, (cy + 0.21) * S],
               fill=shade(base, 0.7), width=max(2, S // 14))
        ell(d, lx + 0.045, cy + 0.22, 0.030, 0.012, S, fill=shade(base, 0.7))
        for tk in range(3):  # toes
            d.line([(lx + 0.045 + (tk - 1) * 0.012) * S, (cy + 0.225) * S,
                    (lx + 0.045 + (tk - 1) * 0.022) * S, (cy + 0.245) * S],
                   fill=shade(base, 0.7), width=max(1, S // 32))
    # eye + ear opening
    ell(d, 0.775, cy - 0.012, 0.013, 0.013, S, fill=(16, 12, 8, 255))
    ell(d, 0.735, cy + 0.005, 0.008, 0.010, S, fill=outline)
    return img

# ---------------------------------------------------------------- turtles
def draw_turtle(S, rng, shell, shell_dark, kind):
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    outline = shade(shell, 0.42)
    # tail (snapping: long sawtooth; painted: stub)
    if kind == "snap":
        tail = [(0.20, 0.60), (0.10, 0.635), (0.135, 0.655), (0.075, 0.675), (0.115, 0.695),
                (0.045, 0.72), (0.19, 0.685)]
        poly(d, tail, S, fill=outline)
        tail2 = [(0.20, 0.615), (0.115, 0.645), (0.085, 0.685), (0.19, 0.67)]
        poly(d, tail2, S, fill=shade(shell, 0.8))
    else:
        ell(d, 0.175, 0.635, 0.030, 0.018, S, fill=outline)
    # legs
    for lx, ly in ((0.30, 0.70), (0.68, 0.70)):
        ell(d, lx, ly, 0.045, 0.028, S, fill=outline)
        ell(d, lx, ly - 0.004, 0.034, 0.020, S, fill=shade(shell, 0.75))
    # shell dome
    dome = []
    for i in range(25):
        t = i / 24.0
        x = 0.13 + 0.72 * t
        y = 0.68 - 0.375 * math.sin(math.pi * t) ** 0.8
        if kind == "snap" and t > 0.75:  # jagged rear rim
            y += 0.018 * (1 if i % 2 else -1)
        dome.append((x, y))
    dome += [(0.85, 0.68), (0.13, 0.68)]
    poly(d, dome, S, fill=outline)
    inner = [(0.5 + (x - 0.5) * 0.93, 0.68 - (0.68 - y) * 0.90) for x, y in dome]
    poly(d, inner, S, fill=shell)
    # plastron band
    d.line([0.16 * S, 0.67 * S, 0.82 * S, 0.67 * S], fill=shell_dark, width=max(2, S // 16))
    # scute seams
    for fx in (0.32, 0.50, 0.68):
        d.arc([(fx - 0.15) * S, 0.335 * S, (fx + 0.15) * S, 0.695 * S], start=300, end=60,
              fill=shell_dark, width=max(1, S // 32))
    d.arc([0.16 * S, 0.375 * S, 0.84 * S, 0.715 * S], start=180, end=360,
          fill=shell_dark, width=max(1, S // 32))
    if kind == "painted":  # red marginal marks along the rim
        for i in range(9):
            t = 0.06 + i * 0.11
            x = 0.13 + 0.72 * t
            y = 0.68 - 0.375 * math.sin(math.pi * t) ** 0.8
            ell(d, x, y + 0.012, 0.016, 0.010, S, fill=(208, 84, 44, 255))
    else:  # rough speckle
        speckle(img, rng, [shell_dark], int(S * 0.7), 0.008, 0.018)
        d = ImageDraw.Draw(img)
    # head, poking out to the right
    ell(d, 0.885, 0.585, 0.052, 0.042, S, fill=outline)
    ell(d, 0.885, 0.585, 0.042, 0.033, S, fill=shade(shell, 0.85))
    if kind == "painted":  # yellow head stripes
        for k in range(3):
            yy = 0.562 + k * 0.020
            d.line([0.855 * S, yy * S, 0.918 * S, (yy + 0.004) * S],
                   fill=(236, 214, 120, 255), width=max(1, S // 32))
    else:  # hooked beak
        d.polygon([0.918 * S, 0.575 * S, 0.945 * S, 0.60 * S, 0.915 * S, 0.605 * S],
                  fill=(196, 188, 168, 255))
    ell(d, 0.895, 0.572, 0.010, 0.010, S, fill=(14, 10, 6, 255))
    return img

# ---------------------------------------------------------------- tadpoles
def draw_tadpole(S, rng, base, spec_cols, big=False, tail="pale", belly=None,
                 warts=False, eyes_apart=False, pointed=False, fill=1.0):
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    outline = shade(base, 0.45)
    brx = (0.235 if big else 0.155) * fill
    bry = brx * 0.88
    bcx, bcy = 0.5 - brx * 0.9, 0.50
    # tail fin on an overlay (translucent by nature) with an opaque core
    fin = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    fd = ImageDraw.Draw(fin)
    n = 30
    spine = []
    for i in range(n + 1):
        t = i / n
        x = bcx + brx * 0.75 + t * (0.94 - (bcx + brx * 0.75))
        y = bcy + 0.10 * math.sin(t * math.pi * 1.6) * (0.4 + 0.6 * t)
        wdt = (0.085 if big else 0.105) * math.sin(math.pi * min(t * 1.12, 1.0)) ** 0.7
        if tail == "thin":
            wdt *= 0.45
        spine.append((x, y, wdt))
    top = [(x, y - w) for x, y, w in spine]
    bot = [(x, y + w) for x, y, w in reversed(spine)]
    if tail == "gradient":  # crimson at the body to yellow at the tip
        for i in range(n):
            t = i / n
            c = (int(214 + (238 - 214) * t), int(52 + (196 - 52) * t), int(44 + (64 - 44) * t), 165)
            seg = [top[i], top[i + 1], bot[n - 1 - i], bot[n - i]]
            fd.polygon([(x * S, y * S) for x, y in seg], fill=c)
    else:
        fincol = {"pale": (206, 210, 186, 150), "redtip": (208, 200, 178, 150),
                  "dark": (120, 116, 104, 150)}.get(tail, (206, 210, 186, 150))
        fd.polygon([(x * S, y * S) for x, y in top + bot], fill=fincol)
        if tail == "redtip":  # bright red-orange rear half, dotted black
            seg_top = [(x, y - w) for x, y, w in spine[15:]]
            seg_bot = [(x, y + w) for x, y, w in reversed(spine[15:])]
            fd.polygon([(x * S, y * S) for x, y in seg_top + seg_bot], fill=(228, 96, 46, 190))
            for _ in range(int(S * 0.35)):
                i = rng.randrange(16, n)
                x, y, w = spine[i]
                yy = y + (rng.random() * 2 - 1) * w * 0.8
                r = max(1.0, S * 0.010)
                fd.ellipse([x * S - r, yy * S - r, x * S + r, yy * S + r], fill=(24, 20, 16, 230))
    # opaque tail muscle core
    core = [(x, y) for x, y, _ in spine]
    fd.line([(x * S, y * S) for x, y in core], fill=shade(base, 0.8)[:3] + (255,),
            width=max(1, int(S * 0.02)))
    img.alpha_composite(fin)
    d = ImageDraw.Draw(img)
    # body
    if pointed:  # pickerel: pointed face -- ellipse + snout wedge
        ell(d, bcx, bcy, brx + 0.012, bry + 0.010, S, fill=outline)
        ell(d, bcx, bcy, brx, bry, S, fill=base)
        sn = [(bcx - brx * 0.75, bcy - bry * 0.45), (bcx - brx * 1.45, bcy + 0.01),
              (bcx - brx * 0.75, bcy + bry * 0.45)]
        poly(d, sn, S, fill=outline)
        sn2 = [(x + 0.008, y) for x, y in sn]
        poly(d, sn2, S, fill=base)
    else:
        ell(d, bcx, bcy, brx + 0.012, bry + 0.010, S, fill=outline)
        ell(d, bcx, bcy, brx, bry, S, fill=base)
    if belly:
        ell(d, bcx, bcy + bry * 0.42, brx * 0.72, bry * 0.5, S, fill=belly)
    if spec_cols:
        speckle(img, rng, spec_cols, int(S * (1.6 if big else 1.1)), 0.008, 0.020)
        d = ImageDraw.Draw(img)
    if warts:
        for _ in range(int(S * 0.30)):
            x = bcx + (rng.random() * 2 - 1) * brx * 0.8
            y = bcy + (rng.random() * 2 - 1) * bry * 0.8
            r = S * (0.012 + rng.random() * 0.012)
            d.ellipse([x * S - r, y * S - r, x * S + r, y * S + r], fill=shade(base, 0.7))
    # eyes: front-top of the body; wide-set for the spadefoot tadpole
    gap = 0.52 if eyes_apart else 0.34
    for sgn in (-1, 1):
        ex = bcx - brx * 0.25 + sgn * brx * gap
        ey = bcy - bry * 0.52
        er = max(1.2, S * 0.013)
        d.ellipse([ex * S - er, ey * S - er, ex * S + er, ey * S + er], fill=(18, 14, 8, 255))
        d.ellipse([ex * S - er * 0.4, ey * S - er * 0.4, ex * S + er * 0.1, ey * S + er * 0.1],
                  fill=(240, 238, 220, 255))
    return img

# ---------------------------------------------------------------- frogs
def frog_silhouette(S, fill, big_head=False, squat=False):
    """Return (mask, shapes) for the squat front pose."""
    cx = 0.5
    cyb = 0.60 if not squat else 0.64
    f = fill
    hrx = 0.27 * f * (1.18 if big_head else 1.0)
    shapes = []
    for sgn in (-1, 1):  # folded hind legs + feet
        shapes.append(rellipse(cx + sgn * 0.295 * f, cyb + 0.08, 0.165 * f, 0.235 * f, sgn * 0.35))
        shapes.append(rellipse(cx + sgn * 0.345 * f, 0.905, 0.115 * f, 0.048 * f, sgn * 0.15))
    shapes.append(rellipse(cx, cyb, 0.30 * f, 0.295 * f * (0.82 if squat else 1.0)))
    shapes.append(rellipse(cx, cyb - 0.235 * f, hrx, 0.20 * f * (1.12 if big_head else 1.0)))
    for sgn in (-1, 1):  # eye bumps
        shapes.append(rellipse(cx + sgn * 0.128 * f, cyb - 0.415 * f, 0.062 * f, 0.058 * f))
    mask = Image.new("L", (S, S), 0)
    md = ImageDraw.Draw(mask)
    for pts in shapes:
        md.polygon([(x * S, y * S) for x, y in pts], fill=255)
    return mask, cyb, f

def draw_frog(S, rng, base, belly=None, pattern=None, pupil="round",
              iris=(24, 18, 10, 255), warty=False, fill=0.92, big_head=False,
              squat=False, parotoid=False, leg_scale=1.0):
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    outline = shade(base, 0.42)
    mask, cyb, f = frog_silhouette(S, fill * leg_scale if False else fill, big_head, squat)
    cx = 0.5
    # outline: dilated mask
    dil = mask.filter(ImageFilter.MaxFilter(max(3, (S // 16) | 1)))
    base_img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    base_img.paste(Image.new("RGBA", (S, S), outline), (0, 0), dil)
    base_img.paste(Image.new("RGBA", (S, S), base), (0, 0), mask)
    img.alpha_composite(base_img)
    d = ImageDraw.Draw(img)
    # hind-leg shading (darker folded legs at the sides)
    leg_layer = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    ld = ImageDraw.Draw(leg_layer)
    for sgn in (-1, 1):
        pts = rellipse(cx + sgn * 0.295 * f, cyb + 0.08, 0.165 * f * leg_scale, 0.235 * f, sgn * 0.35)
        ld.polygon([(x * S, y * S) for x, y in pts], fill=shade(base, 0.78))
        pts = rellipse(cx + sgn * 0.345 * f, 0.905, 0.115 * f, 0.048 * f, sgn * 0.15)
        ld.polygon([(x * S, y * S) for x, y in pts], fill=shade(base, 0.78))
    clipped(img, leg_layer)
    d = ImageDraw.Draw(img)
    if belly:
        bl = Image.new("RGBA", (S, S), (0, 0, 0, 0))
        bd = ImageDraw.Draw(bl)
        bd.ellipse([(cx - 0.185 * f) * S, (cyb + 0.02) * S, (cx + 0.185 * f) * S,
                    (cyb + 0.36 * f) * S], fill=belly)
        clipped(img, bl)
        d = ImageDraw.Draw(img)
    if parotoid:  # swollen glands behind the eyes (toads)
        for sgn in (-1, 1):
            ell(d, cx + sgn * 0.215 * f, cyb - 0.335 * f, 0.068 * f, 0.042 * f, S,
                fill=shade(base, 0.72))
    if warty:
        speckle(img, rng, [shade(base, 0.7), shade(base, 1.18)], int(S * 1.1), 0.010, 0.026)
        d = ImageDraw.Draw(img)
    if pattern:
        pattern(img, d, S, rng, f, cyb)
        d = ImageDraw.Draw(img)
    # front arms
    for sgn in (-1, 1):
        pts = rellipse(cx + sgn * 0.145 * f, cyb + 0.235 * f, 0.034 * f, 0.095 * f, sgn * 0.12)
        poly(d, pts, S, fill=shade(base, 0.8))
        for tk in range(3):
            tx = cx + sgn * (0.145 + (tk - 1) * 0.030) * f
            ell(d, tx, cyb + 0.335 * f, 0.011 * f, 0.011 * f, S, fill=shade(base, 0.8))
    # mouth: wide shallow arc
    mw = 0.185 * f * (1.15 if big_head else 1.0)
    d.arc([(cx - mw) * S, (cyb - 0.335 * f) * S, (cx + mw) * S, (cyb - 0.075 * f) * S],
          start=15, end=165, fill=shade(base, 0.4), width=max(1, S // 24))
    # nostrils
    for sgn in (-1, 1):
        er = max(1.0, S * 0.008)
        ex, ey = (cx + sgn * 0.055 * f) * S, (cyb - 0.315 * f) * S
        d.ellipse([ex - er, ey - er, ex + er, ey + er], fill=shade(base, 0.4))
    # eyes on the bumps
    for sgn in (-1, 1):
        ex, ey = cx + sgn * 0.128 * f, cyb - 0.415 * f
        er = 0.040 * f
        ell(d, ex, ey, er, er, S, fill=iris)
        if pupil == "vertical":
            d.line([ex * S, (ey - er * 0.7) * S, ex * S, (ey + er * 0.7) * S],
                   fill=(10, 8, 4, 255), width=max(1, int(S * 0.016)))
        elif pupil == "sleepy":  # half-lidded
            ell(d, ex, ey - er * 0.35, er, er * 0.62, S, fill=shade(base, 0.85))
            d.line([(ex - er) * S, ey * S, (ex + er) * S, ey * S], fill=(12, 10, 6, 255),
                   width=max(1, S // 32))
        elif pupil == "vacant":  # pale, unfocused
            ell(d, ex, ey, er * 0.55, er * 0.55, S, fill=(196, 200, 178, 255))
            ell(d, ex, ey, er * 0.22, er * 0.22, S, fill=(60, 58, 48, 255))
        else:
            ell(d, ex, ey, er * 0.52, er * 0.52, S, fill=(12, 10, 6, 255))
            ell(d, ex - er * 0.18, ey - er * 0.20, er * 0.16, er * 0.16, S,
                fill=(245, 244, 230, 255))
    return img

# --- frog pattern / extra-feature painters (drawn after the base body) ---
def pat_spots(spot, halo=None, n=9, rmin=0.02, rmax=0.045):
    def fn(img, d, S, rng, f, cyb):
        layer = Image.new("RGBA", (S, S), (0, 0, 0, 0))
        ld = ImageDraw.Draw(layer)
        for _ in range(n):
            x = 0.5 + (rng.random() * 2 - 1) * 0.30 * f
            y = cyb - 0.30 * f + rng.random() * 0.62 * f
            r = (rmin + rng.random() * (rmax - rmin)) * f
            if halo:
                ld.ellipse([(x - r * 1.45) * S, (y - r * 1.45) * S, (x + r * 1.45) * S,
                            (y + r * 1.45) * S], fill=halo)
            ld.ellipse([(x - r) * S, (y - r) * S, (x + r) * S, (y + r) * S], fill=spot)
        clipped(img, layer)
    return fn

def pat_mottle(cols, n=260, rmin=0.006, rmax=0.016):
    def fn(img, d, S, rng, f, cyb):
        speckle(img, rng, cols, int(n * S / 32), rmin, rmax)
    return fn

def pat_static():
    def fn(img, d, S, rng, f, cyb):
        layer = Image.new("RGBA", (S, S), (0, 0, 0, 0))
        ld = ImageDraw.Draw(layer)
        for _ in range(26):  # fragmented pigment collages
            x = 0.5 + (rng.random() * 2 - 1) * 0.30 * f
            y = cyb - 0.32 * f + rng.random() * 0.60 * f
            w = (0.02 + rng.random() * 0.05) * f
            h = (0.012 + rng.random() * 0.03) * f
            c = [(58, 58, 60, 255), (96, 98, 100, 255), (30, 30, 34, 255)][rng.randrange(3)]
            ld.rectangle([(x - w) * S, (y - h) * S, (x + w) * S, (y + h) * S], fill=c)
        clipped(img, layer)
        speckle(img, rng, [(40, 40, 44, 255), (210, 212, 208, 255)], int(S * 1.4), 0.005, 0.012)
    return fn

def pat_xband(col):
    def fn(img, d, S, rng, f, cyb):
        layer = Image.new("RGBA", (S, S), (0, 0, 0, 0))
        ld = ImageDraw.Draw(layer)
        w = max(2, int(S * 0.045 * f))
        ld.line([0.30 * S, (cyb - 0.34 * f) * S, 0.70 * S, (cyb + 0.18 * f) * S], fill=col, width=w)
        ld.line([0.70 * S, (cyb - 0.34 * f) * S, 0.30 * S, (cyb + 0.18 * f) * S], fill=col, width=w)
        clipped(img, layer)
    return fn

def pat_eye_mask(col):
    def fn(img, d, S, rng, f, cyb):
        layer = Image.new("RGBA", (S, S), (0, 0, 0, 0))
        ld = ImageDraw.Draw(layer)
        ey = cyb - 0.415 * f
        ld.polygon([(0.24) * S, (ey - 0.045 * f) * S, (0.76) * S, (ey - 0.045 * f) * S,
                    (0.72) * S, (ey + 0.055 * f) * S, (0.28) * S, (ey + 0.055 * f) * S], fill=col)
        clipped(img, layer)
    return fn

def pat_palette_patch():
    hues = [(232, 64, 160, 255), (48, 200, 220, 255), (250, 214, 52, 255),
            (120, 224, 80, 255), (250, 128, 40, 255), (150, 90, 230, 255)]
    def fn(img, d, S, rng, f, cyb):
        layer = Image.new("RGBA", (S, S), (0, 0, 0, 0))
        ld = ImageDraw.Draw(layer)
        for _ in range(8):
            x = 0.5 + (rng.random() * 2 - 1) * 0.28 * f
            y = cyb - 0.28 * f + rng.random() * 0.52 * f
            w = (0.035 + rng.random() * 0.03) * f
            ld.rectangle([(x - w) * S, (y - w * 0.8) * S, (x + w) * S, (y + w * 0.8) * S],
                         fill=hues[rng.randrange(len(hues))])
        clipped(img, layer)
    return fn

def pat_gills(col=(196, 138, 128, 255)):
    def fn(img, d, S, rng, f, cyb):
        for sgn in (-1, 1):  # feathery external gill tufts at the neck
            gx, gy = 0.5 + sgn * 0.265 * f, cyb - 0.16 * f
            for k in range(5):
                a = (k - 2) * 0.28
                ex = gx + sgn * (0.045 + 0.02 * abs(k - 2)) * f
                ey = gy + a * 0.09 * f
                d.line([gx * S, gy * S, ex * S, ey * S], fill=col, width=max(1, S // 24))
                d.line([gx * S, gy * S, (ex - sgn * 0.015 * f) * S, (ey + 0.02 * f) * S],
                       fill=col, width=max(1, S // 32))
    return fn

def pat_gill_bulges(base):
    def fn(img, d, S, rng, f, cyb):
        for sgn in (-1, 1):  # gill masses bulging the flesh under the mouth
            ell(d, 0.5 + sgn * 0.115 * f, cyb - 0.055 * f, 0.075 * f, 0.055 * f, S,
                fill=shade(base, 0.82))
    return fn

def pat_glow(neck=False):
    def fn(img, d, S, rng, f, cyb):
        ov = Image.new("RGBA", (S, S), (0, 0, 0, 0))
        od = ImageDraw.Draw(ov)
        spots = []
        for _ in range(9):
            x = 0.5 + (rng.random() * 2 - 1) * 0.26 * f
            y = cyb - 0.26 * f + rng.random() * 0.45 * f
            spots.append((x, y))
        if neck:  # taser tongue: glow concentrated at neck and mouth
            for k in range(5):
                spots.append((0.5 + (k - 2) * 0.075 * f, cyb - 0.10 * f + abs(k - 2) * 0.012 * f))
        for x, y in spots:
            r = S * 0.030 * f
            od.ellipse([x * S - r, y * S - r, x * S + r, y * S + r], fill=(110, 230, 240, 70))
            r2 = r * 0.42
            od.ellipse([x * S - r2, y * S - r2, x * S + r2, y * S + r2], fill=(190, 250, 255, 235))
        img.alpha_composite(ov)
    return fn

def pat_sheen():
    def fn(img, d, S, rng, f, cyb):
        ov = Image.new("RGBA", (S, S), (0, 0, 0, 0))
        od = ImageDraw.Draw(ov)
        for k in range(4):  # greasy streaks
            x = 0.5 + (rng.random() * 2 - 1) * 0.22 * f
            y = cyb - 0.25 * f + rng.random() * 0.4 * f
            od.line([x * S, y * S, (x + 0.07 * f) * S, (y + 0.05 * f) * S],
                    fill=(240, 244, 220, 60), width=max(2, S // 12))
        img.alpha_composite(ov)
        # stink wisps rising off the back
        for k in range(3):
            x = 0.5 + (k - 1) * 0.13 * f
            pts = [(x + 0.02 * math.sin(t * 5 + k), cyb - 0.46 * f - t * 0.09) for t in
                   (0.0, 0.25, 0.5, 0.75, 1.0)]
            od.line([(px * S, py * S) for px, py in pts], fill=(150, 190, 90, 80),
                    width=max(1, S // 24))
        img.alpha_composite(ov)
    return fn

def pat_filaments(leaf=(96, 138, 66, 255), stem=(80, 110, 58, 255)):
    def fn(img, d, S, rng, f, cyb):
        for _ in range(int(S * 0.55)):  # ghillie filaments around the silhouette
            a = rng.random() * math.pi * 2
            rx = 0.30 * f * math.cos(a) * 1.12
            ry = 0.36 * f * math.sin(a)
            x, y = 0.5 + rx, cyb - 0.05 + ry
            ln = 0.03 + rng.random() * 0.05
            ex, ey = x + rx * ln * 2.2, y + ry * ln * 2.2 - 0.012
            d.line([x * S, y * S, ex * S, ey * S], fill=stem, width=max(1, S // 32))
            if rng.random() < 0.5:
                ell(d, ex, ey, 0.016, 0.008, S, fill=leaf)
    return fn

def pat_extra_legs(base, tall=False):
    def fn(img, d, S, rng, f, cyb):
        col = shade(base, 0.75)
        if tall:  # wood worm: spindly legs rising high, then bending down
            for k in range(6):
                sgn = -1 if k % 2 == 0 else 1
                row = k // 2
                hx = 0.5 + sgn * (0.20 + row * 0.045) * f
                hy = cyb + (0.02 + row * 0.13) * f
                kx, ky = hx + sgn * 0.10 * f, hy - 0.30 * f
                fx, fy = kx + sgn * 0.05 * f, cyb + 0.36 * f
                d.line([hx * S, hy * S, kx * S, ky * S], fill=col, width=max(1, S // 24))
                d.line([kx * S, ky * S, fx * S, fy * S], fill=col, width=max(1, S // 24))
        else:  # leo worm: a dozen extra legs, many malformed
            for k in range(11):
                sgn = -1 if k % 2 == 0 else 1
                row = k // 2
                hx = 0.5 + sgn * (0.24 + rng.random() * 0.06) * f
                hy = cyb - 0.05 * f + row * 0.085 * f
                ln = (0.05 + rng.random() * 0.075) * (0.45 if k % 3 == 0 else 1.0)
                ex = hx + sgn * ln * f
                ey = hy + (rng.random() * 0.10 - 0.02) * f
                d.line([hx * S, hy * S, ex * S, ey * S], fill=col,
                       width=max(1, int(S * 0.018)))
                ell(d, ex, ey, 0.010, 0.010, S, fill=col)
    return fn

def pat_tusks():
    def fn(img, d, S, rng, f, cyb):
        for sgn in (-1, 1):  # dense tusk-like growths from face and jaw
            for k, (dx, ln, curv) in enumerate(((0.16, 0.13, 0.05), (0.225, 0.10, 0.07))):
                bx = 0.5 + sgn * dx * f
                by = cyb - 0.115 * f + k * 0.045 * f
                tipx = bx + sgn * curv * f
                tipy = by - ln * f
                midx = bx + sgn * curv * f * 0.4
                midy = by - ln * f * 0.55
                wdt = 0.016 * f
                d.polygon([(bx - wdt) * S, by * S, (bx + wdt) * S, by * S,
                           tipx * S, tipy * S], fill=(228, 220, 196, 255))
                d.line([bx * S, by * S, midx * S, midy * S, tipx * S, tipy * S],
                       fill=(240, 234, 214, 255), width=max(1, S // 32))
        # hog-like blunt muzzle + nostrils
        ell(d, 0.5, cyb - 0.135 * f, 0.105 * f, 0.062 * f, S, fill=(150, 128, 104, 255))
        for sgn in (-1, 1):
            ell(d, 0.5 + sgn * 0.038 * f, cyb - 0.135 * f, 0.013 * f, 0.017 * f, S,
                fill=(70, 56, 42, 255))
    return fn

def pat_beak(bone=(216, 204, 170, 255)):
    def fn(img, d, S, rng, f, cyb):
        # bony jaw plates grown out into a beak
        up = [(0.5 - 0.155 * f, cyb - 0.175 * f), (0.5 + 0.155 * f, cyb - 0.175 * f),
              (0.5 + 0.055 * f, cyb - 0.045 * f), (0.5 - 0.055 * f, cyb - 0.045 * f)]
        poly(d, up, S, fill=bone)
        lo = [(0.5 - 0.115 * f, cyb - 0.035 * f), (0.5 + 0.115 * f, cyb - 0.035 * f),
              (0.5 + 0.04 * f, cyb + 0.045 * f), (0.5 - 0.04 * f, cyb + 0.045 * f)]
        poly(d, lo, S, fill=shade(bone, 0.82))
        d.line([0.5 * S, (cyb - 0.178 * f) * S, 0.5 * S, (cyb - 0.05 * f) * S],
               fill=shade(bone, 0.7), width=max(1, S // 32))
        for sgn in (-1, 1):  # bony rings shielding the eyes
            ex, ey = 0.5 + sgn * 0.128 * f, cyb - 0.415 * f
            d.ellipse([(ex - 0.058 * f) * S, (ey - 0.058 * f) * S, (ex + 0.058 * f) * S,
                       (ey + 0.058 * f) * S], outline=bone, width=max(2, S // 16))
    return fn

def pat_throat_sac(pale=(232, 220, 188, 255)):
    def fn(img, d, S, rng, f, cyb):
        # open wailing mouth + inflated vocal sac beneath it
        ell(d, 0.5, cyb - 0.145 * f, 0.085 * f, 0.052 * f, S, fill=(52, 30, 26, 255))
        ell(d, 0.5, cyb - 0.155 * f, 0.062 * f, 0.032 * f, S, fill=(96, 44, 40, 255))
        ell(d, 0.5, cyb + 0.005 * f, 0.125 * f, 0.095 * f, S, fill=pale)
        ell(d, 0.5, cyb - 0.01 * f, 0.095 * f, 0.065 * f, S, fill=shade(pale, 1.06))
    return fn

def pat_toe_pads(base):
    def fn(img, d, S, rng, f, cyb):
        for sgn in (-1, 1):
            for tk in range(3):  # round treefrog toe pads on the hind feet
                tx = 0.5 + sgn * (0.345 + (tk - 1) * 0.045) * f
                ell(d, tx, cyb + 0.315 * f, 0.020 * f, 0.020 * f, S, fill=shade(base, 1.22))
    return fn

def pat_tail_paddle(base, spot):
    def fn(img, d, S, rng, f, cyb):
        # broad paddle tail sweeping out below the body (half its mass)
        pts = [(0.40, cyb + 0.24 * f), (0.60, cyb + 0.24 * f), (0.65, cyb + 0.33 * f),
               (0.57, cyb + 0.42 * f), (0.43, cyb + 0.42 * f), (0.35, cyb + 0.33 * f)]
        poly(d, pts, S, fill=shade(base, 0.5))
        pts2 = [(0.5 + (x - 0.5) * 0.88, cyb + 0.24 * f + (y - (cyb + 0.24 * f)) * 0.85)
                for x, y in pts]
        poly(d, pts2, S, fill=base)
        for _ in range(7):
            x = 0.5 + (rng.random() * 2 - 1) * 0.11
            y = cyb + (0.27 + rng.random() * 0.11) * f
            r = S * 0.012
            d.ellipse([x * S - r, y * S - r, x * S + r, y * S + r], fill=spot)
    return fn

def pat_tail_stub(base):
    def fn(img, d, S, rng, f, cyb):
        pts = [(0.455, cyb + 0.30 * f), (0.545, cyb + 0.30 * f), (0.5, cyb + 0.44 * f)]
        poly(d, pts, S, fill=shade(base, 0.8))
    return fn

def combine(*fns):
    def fn(img, d, S, rng, f, cyb):
        for g in fns:
            g(img, d, S, rng, f, cyb)
            d = ImageDraw.Draw(img)
    return fn

# ---------------------------------------------------------------- jobs
N32 = "pngs_normal_32x32/monsters"
L64 = "pngs_large_64x64/monsters"
G96 = "pngs_giant_96x96/monsters"

def J(mid, sheet, size, fn):
    return (f"{sheet}/{mid}/{mid}", size, fn)

LEO_SPOTS = pat_spots((74, 56, 32, 255), halo=(198, 216, 150, 255), n=10)
JOBS = [
    # snakes
    J("mon_gartersnake", N32, 32, lambda S, r: draw_snake(
        S, r, (110, 104, 62, 255), (70, 66, 40, 255),
        pattern="stripes", pat=(228, 208, 110, 255))),
    J("mon_gartersnake_baby", N32, 32, lambda S, r: draw_snake(
        S, r, (114, 108, 64, 255), (72, 68, 42, 255),
        pattern="stripes", pat=(234, 216, 118, 255), thick=0.046, span=0.78)),
    J("mon_wormsnake", N32, 32, lambda S, r: draw_snake(
        S, r, (122, 84, 66, 255), (80, 54, 42, 255), belly=(233, 163, 152, 255),
        pattern="speckle", pat=(96, 62, 48, 255), thick=0.048, span=0.80)),
    J("mon_wormsnake_baby", N32, 32, lambda S, r: draw_snake(
        S, r, (126, 88, 70, 255), (82, 56, 44, 255), belly=(236, 170, 160, 255),
        thick=0.036, span=0.66)),
    J("mon_rattlesnake_s", N32, 32, lambda S, r: draw_snake(
        S, r, (150, 124, 88, 255), (96, 76, 50, 255), pattern="bands",
        pat=(88, 64, 40, 255), rattle=1, thick=0.048, span=0.78)),
    J("mon_rattlesnake_big_s", N32, 32, lambda S, r: draw_snake(
        S, r, (146, 120, 84, 255), (94, 74, 48, 255), pattern="bands",
        pat=(84, 60, 38, 255), rattle=1, thick=0.062, span=0.82)),
    J("mon_rattlesnake_big", N32, 32, lambda S, r: draw_snake(
        S, r, (128, 110, 84, 255), (82, 68, 50, 255), pattern="bands",
        pat=(70, 52, 34, 255), rattle=5, thick=0.082)),
    J("mon_rattlesnake_mega", G96, 96, lambda S, r: draw_snake(
        S, r, (124, 96, 64, 255), (78, 58, 38, 255), pattern="bands",
        pat=(66, 46, 28, 255), club=True, mega=True, thick=0.085)),
    # skinks
    J("mon_skink_fivelined", N32, 32, lambda S, r: draw_skink(
        S, r, (86, 64, 44, 255), (233, 221, 181, 255))),
    J("mon_skink_fivelined_baby", N32, 32, lambda S, r: draw_skink(
        S, r, (42, 38, 34, 255), (240, 232, 200, 255), tail_col=(74, 116, 224, 255))),
    # turtle hatchlings
    J("mon_turtle_painted_baby", N32, 32, lambda S, r: draw_turtle(
        S, r, (86, 94, 52, 255), (56, 64, 32, 255), "painted")),
    J("mon_turtle_snapping_common_baby", N32, 32, lambda S, r: draw_turtle(
        S, r, (94, 86, 72, 255), (60, 54, 42, 255), "snap")),
    # tadpoles
    J("mon_leo_tadpole", N32, 32, lambda S, r: draw_tadpole(
        S, r, (128, 118, 96, 255), [(84, 76, 60, 255), (168, 158, 134, 255)])),
    J("mon_bullfrog_tadpole", N32, 32, lambda S, r: draw_tadpole(
        S, r, (110, 128, 72, 255), [(70, 84, 44, 255), (150, 164, 104, 255)])),
    J("mon_fowler_tadpole", N32, 32, lambda S, r: draw_tadpole(
        S, r, (74, 68, 60, 255), [(150, 142, 128, 255)], tail="thin")),
    J("mon_freedom_tadpole", N32, 32, lambda S, r: draw_tadpole(
        S, r, (78, 70, 62, 255), [(154, 146, 130, 255)], tail="thin")),
    J("mon_peeper_tadpole", N32, 32, lambda S, r: draw_tadpole(
        S, r, (36, 34, 32, 255), [], tail="dark", fill=0.55)),
    J("mon_gray_tadpole", N32, 32, lambda S, r: draw_tadpole(
        S, r, (122, 112, 98, 255), [(88, 80, 66, 255)], tail="redtip")),
    J("mon_mink_tadpole", N32, 32, lambda S, r: draw_tadpole(
        S, r, (140, 118, 84, 255), [(96, 78, 52, 255), (178, 158, 118, 255)])),
    J("mon_pickerel_tadpole", N32, 32, lambda S, r: draw_tadpole(
        S, r, (196, 178, 96, 255), [(128, 102, 60, 255), (232, 220, 180, 255)],
        pointed=True)),
    J("mon_green_tadpole", N32, 32, lambda S, r: draw_tadpole(
        S, r, (104, 152, 70, 255), [(30, 30, 26, 255)], belly=(228, 208, 110, 255))),
    J("mon_wood_tadpole", N32, 32, lambda S, r: draw_tadpole(
        S, r, (128, 104, 72, 255), [(92, 72, 48, 255), (164, 140, 100, 255)])),
    J("mon_spade_tadpole", N32, 32, lambda S, r: draw_tadpole(
        S, r, (130, 116, 88, 255), [(96, 84, 58, 255)], eyes_apart=True)),
    # giant tadpoles
    J("mon_pattern_bigtad", N32, 32, lambda S, r: draw_tadpole(
        S, r, (124, 116, 100, 255),
        [(16, 16, 16, 255), (238, 238, 232, 255), (70, 66, 60, 255)], big=True)),
    J("mon_strange_bigtad", N32, 32, lambda S, r: draw_tadpole(
        S, r, (100, 146, 70, 255), [(66, 104, 44, 255)], big=True)),
    J("mon_foul_bigtad", N32, 32, lambda S, r: draw_tadpole(
        S, r, (110, 104, 80, 255), [(80, 74, 56, 255)], big=True, tail="thin",
        warts=True)),
    J("mon_odd_bigtad", N32, 32, lambda S, r: draw_tadpole(
        S, r, (134, 120, 92, 255), [(90, 78, 56, 255), (170, 156, 124, 255)],
        big=True, tail="thin")),
    J("mon_vocal_bigtad", N32, 32, lambda S, r: draw_tadpole(
        S, r, (46, 44, 42, 255), [(20, 20, 20, 255)], big=True, tail="dark")),
    J("mon_shift_bigtad", N32, 32, lambda S, r: draw_tadpole(
        S, r, (120, 110, 88, 255), [(88, 78, 58, 255)], big=True, tail="gradient")),
    # frogs & toads, normal size
    J("mon_leo_frog", N32, 32, lambda S, r: draw_frog(
        S, r, (104, 152, 64, 255), belly=(216, 226, 180, 255), pattern=LEO_SPOTS)),
    J("mon_pattern_frog", N32, 32, lambda S, r: draw_frog(
        S, r, (118, 128, 108, 255), belly=(188, 196, 172, 255),
        pattern=combine(pat_static(), pat_gills()))),
    J("mon_fowler_toad", N32, 32, lambda S, r: draw_frog(
        S, r, (128, 108, 78, 255), belly=(224, 214, 188, 255), warty=True,
        parotoid=True, pattern=pat_spots((88, 70, 48, 255), n=7, rmin=0.025, rmax=0.05))),
    J("mon_foul_toad", N32, 32, lambda S, r: draw_frog(
        S, r, (110, 112, 72, 255), belly=(214, 204, 172, 255), warty=True,
        parotoid=True, pattern=combine(pat_spots((80, 80, 50, 255), n=6), pat_sheen()))),
    J("mon_odd_toad", N32, 32, lambda S, r: draw_frog(
        S, r, (116, 100, 70, 255), belly=(206, 192, 158, 255), warty=True,
        parotoid=True, big_head=True, pattern=pat_tail_stub((116, 100, 70, 255)))),
    J("mon_peeper_frog", N32, 32, lambda S, r: draw_frog(
        S, r, (188, 160, 110, 255), belly=(232, 220, 188, 255), fill=0.66,
        pattern=pat_xband((110, 84, 52, 255)))),
    J("mon_gray_frog", N32, 32, lambda S, r: draw_frog(
        S, r, (168, 170, 164, 255), belly=(214, 216, 208, 255),
        pattern=combine(pat_mottle([(128, 130, 124, 255), (188, 190, 182, 255)]),
                        pat_toe_pads((168, 170, 164, 255))))),
    J("mon_shift_frog", N32, 32, lambda S, r: draw_frog(
        S, r, (150, 146, 128, 255), belly=(198, 196, 178, 255),
        pattern=combine(pat_mottle([(104, 92, 70, 255), (120, 124, 96, 255)]),
                        pat_filaments()))),
    J("mon_mink_frog", N32, 32, lambda S, r: draw_frog(
        S, r, (126, 178, 92, 255), belly=(196, 224, 160, 255),
        pattern=pat_mottle([(24, 30, 20, 255)], n=340))),
    J("mon_tail_frog", N32, 32, lambda S, r: draw_frog(
        S, r, (96, 150, 70, 255), belly=(200, 220, 160, 255),
        pattern=combine(pat_spots((110, 84, 48, 255), n=8),
                        pat_tail_paddle((96, 150, 70, 255), (110, 84, 48, 255))))),
    J("mon_palette_frog", N32, 32, lambda S, r: draw_frog(
        S, r, (176, 148, 104, 255), belly=(226, 206, 168, 255),
        pattern=pat_palette_patch())),
    J("mon_wood_frog", N32, 32, lambda S, r: draw_frog(
        S, r, (160, 128, 88, 255), belly=(236, 226, 196, 255),
        pattern=pat_eye_mask((84, 60, 38, 255)))),
    J("mon_spade_frog", N32, 32, lambda S, r: draw_frog(
        S, r, (140, 124, 84, 255), belly=(206, 192, 150, 255), pupil="vertical",
        squat=True, pattern=pat_mottle([(110, 96, 60, 255), (168, 160, 104, 255)]))),
    J("mon_leo_worm", N32, 32, lambda S, r: draw_frog(
        S, r, (104, 152, 64, 255), belly=(216, 226, 180, 255),
        pattern=combine(LEO_SPOTS, pat_extra_legs((104, 152, 64, 255))))),
    J("mon_wood_worm", N32, 32, lambda S, r: draw_frog(
        S, r, (160, 128, 88, 255), belly=(236, 226, 196, 255),
        pattern=combine(pat_eye_mask((84, 60, 38, 255)),
                        pat_extra_legs((160, 128, 88, 255), tall=True)))),
    # big frogs (still <100 L, on the 32 sheet)
    J("mon_pattern_bigfrog", N32, 32, lambda S, r: draw_frog(
        S, r, (196, 204, 190, 255), belly=(226, 230, 218, 255), fill=0.95,
        pattern=combine(pat_static(), pat_gill_bulges((196, 204, 190, 255)),
                        pat_glow()))),
    J("mon_foul_bigfrog", N32, 32, lambda S, r: draw_frog(
        S, r, (104, 104, 62, 255), belly=(150, 148, 100, 255), warty=True,
        parotoid=True, fill=0.95, pattern=pat_sheen())),
    J("mon_jumbo_bigfrog", N32, 32, lambda S, r: draw_frog(
        S, r, (110, 140, 74, 255), belly=(196, 214, 160, 255), fill=0.95,
        pattern=pat_toe_pads((110, 140, 74, 255)))),
    J("mon_vocal_bigfrog", N32, 32, lambda S, r: draw_frog(
        S, r, (178, 152, 102, 255), belly=(214, 196, 150, 255), fill=0.95,
        pattern=combine(pat_mottle([(140, 116, 72, 255)]), pat_throat_sac(),
                        pat_toe_pads((178, 152, 102, 255))))),
    J("mon_shift_bigfrog", N32, 32, lambda S, r: draw_frog(
        S, r, (158, 164, 140, 255), belly=(204, 210, 188, 255), fill=0.95,
        pattern=combine(pat_mottle([(120, 128, 100, 255), (176, 168, 130, 255)]),
                        pat_filaments(), pat_toe_pads((158, 164, 140, 255))))),
    # giganfrogs (64x64)
    J("mon_strange_giganfrog", L64, 64, lambda S, r: draw_frog(
        S, r, (120, 180, 84, 255), belly=(208, 230, 170, 255), fill=0.95,
        iris=(216, 152, 52, 255), pattern=pat_mottle([(88, 140, 58, 255)], n=160))),
    J("mon_odd_giganfrog", L64, 64, lambda S, r: draw_frog(
        S, r, (124, 104, 72, 255), belly=(196, 178, 142, 255), warty=True,
        parotoid=True, pupil="sleepy", fill=0.95)),
    J("mon_pattern_giganfrog", L64, 64, lambda S, r: draw_frog(
        S, r, (170, 190, 196, 255), belly=(214, 228, 230, 255), fill=0.95,
        pattern=combine(pat_static(), pat_glow(neck=True)))),
    J("mon_foul_giganfrog", L64, 64, lambda S, r: draw_frog(
        S, r, (118, 106, 84, 255), belly=(186, 172, 142, 255), warty=True,
        parotoid=True, fill=0.95, pattern=pat_tusks())),
    # megafrogs
    J("mon_foul_megafrog", L64, 64, lambda S, r: draw_frog(
        S, r, (116, 108, 70, 255), belly=(188, 178, 138, 255), warty=True,
        parotoid=True, fill=0.95, pattern=pat_beak())),
    J("mon_strange_megafrog", G96, 96, lambda S, r: draw_frog(
        S, r, (96, 160, 72, 255), belly=(190, 220, 150, 255), pupil="vacant",
        fill=0.98, pattern=pat_mottle([(70, 124, 50, 255)], n=180))),
    J("mon_odd_megafrog", G96, 96, lambda S, r: draw_frog(
        S, r, (120, 100, 68, 255), belly=(190, 170, 132, 255), warty=True,
        parotoid=True, pupil="sleepy", fill=0.98)),
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
    # contact-sheet preview: uniform 192px cells (32x6, 64x3, 96x2)
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
    out = os.path.join(BASE, "preview_reptiles.png")
    sheet.convert("RGB").save(out)
    print("wrote", out, sheet.size)

if __name__ == "__main__":
    main()
