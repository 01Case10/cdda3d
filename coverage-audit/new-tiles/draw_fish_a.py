#!/usr/bin/env python3
"""
Draw the missing fish sprites, first half (alphabetical) of the fish.json
missing set, from their game descriptions.

Sources (fork-src/cdda/data/json/monsters/fish.json, CDDA 6a3e59e0).
Almost every entry inherits from a size-class abstract (mon_fish_tiny /
_small / _medium / _large / _huge, mon_fry, mon_giant_crayfish); names,
descriptions, colors and volumes below are the copy-from-resolved values.
None of the 104 missing fish entries carries an explicit "size" field, so
per the batch rule (no size -> normal sheet) all 52 sprites of this half
land in pngs_normal_32x32 at 32x32.  (Volumes still span 3 ml for the fry
to 625 L for the colossal crayfish; the sheet rule, not the volume, is
what places them.)

Design: profile view, head left, flat UltiCa style -- dark outline,
back/flank/belly gradient with one highlight band, per-species pattern
(spots, bars, stripes, mottle, chain-link, scutes) clipped to the body
silhouette.  Patterning follows each entry's description first and the
real fish's canonical markings where the description is thin:
  blackfish "dark, mottled" - charcoal + pale mottle; bluefin tuna "dark
  blue back and silvery sides" + finlets; bluefish "blue-green" + teeth;
  haddock "dark lateral line and a black blotch above the pectoral fin";
  channel catfish "forked tail and long whiskers"; carp "golden-yellow";
  muskellunge/pike/pickerel - rear-set fins, duckbill jaws, bars / bean
  spots / chain-link reticulation; sturgeon - armored scute rows and a
  shark-like heterocercal tail; flounder/halibut - flatfish drawn from
  the eyed side; the two crayfish are crustaceans, not fish (segmented
  abdomen, tail fan, pincer claws, long antennae; mega is "enormous
  mutated gray", the larva a pale juvenile); mon_fry_blackfish is a
  "minuscule half-translucent fish larva" -- translucency faked with
  pale opaque colors, per the no-semi-transparent-solids rule.
Output is UltiCa decomposed layout (png + tile-entry json per folder).
This script only ever writes its own 52 folders, itself, and its preview.
"""
from PIL import Image, ImageDraw
import json, math, os, zlib

BASE = os.path.dirname(os.path.abspath(__file__))
OUTLINE = (24, 22, 18, 255)

# ---------------------------------------------------------------- colors
def shade(c, f):
    return tuple(max(0, min(255, int(v * f))) for v in c[:3]) + (255,)

# ---------------------------------------------------------------- geometry
def inside(px, py, poly):
    """Ray-casting point-in-polygon (poly in unit coords, point in unit)."""
    n = len(poly); hit = False; j = n - 1
    for i in range(n):
        xi, yi = poly[i]; xj, yj = poly[j]
        if ((yi > py) != (yj > py)) and (px < (xj - xi) * (py - yi) / (yj - yi) + xi):
            hit = not hit
        j = i
    return hit

def interp(pts, x):
    """Linear y-interpolation over a sorted (x, y) point list."""
    if x <= pts[0][0]:
        return pts[0][1]
    for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
        if x <= x1:
            return y0 + (y1 - y0) * (x - x0) / (x1 - x0)
    return pts[-1][1]

# Body plans: (top edge points, bottom edge points), head at left.
PLANS = {
    "std": (
        [(0.04, 0.50), (0.09, 0.43), (0.20, 0.35), (0.36, 0.315), (0.54, 0.33), (0.68, 0.385), (0.78, 0.455)],
        [(0.78, 0.565), (0.68, 0.635), (0.52, 0.695), (0.36, 0.705), (0.22, 0.675), (0.11, 0.605), (0.04, 0.545)]),
    "deep": (
        [(0.10, 0.50), (0.15, 0.38), (0.26, 0.26), (0.43, 0.215), (0.60, 0.25), (0.72, 0.35), (0.79, 0.455)],
        [(0.79, 0.565), (0.71, 0.70), (0.57, 0.805), (0.42, 0.83), (0.28, 0.775), (0.17, 0.65), (0.10, 0.545)]),
    "long": (
        [(0.03, 0.475), (0.09, 0.435), (0.22, 0.385), (0.44, 0.37), (0.64, 0.39), (0.77, 0.445)],
        [(0.77, 0.565), (0.64, 0.605), (0.44, 0.625), (0.26, 0.615), (0.12, 0.565), (0.03, 0.515)]),
    "duck": (  # pike family: flat duckbill jaw
        [(0.02, 0.46), (0.16, 0.435), (0.26, 0.39), (0.46, 0.375), (0.65, 0.395), (0.77, 0.445)],
        [(0.77, 0.565), (0.65, 0.605), (0.46, 0.62), (0.28, 0.61), (0.16, 0.55), (0.02, 0.525)]),
    "needle": (  # gar: very long thin jaw
        [(0.01, 0.478), (0.20, 0.455), (0.30, 0.41), (0.50, 0.395), (0.66, 0.41), (0.77, 0.45)],
        [(0.77, 0.56), (0.66, 0.595), (0.48, 0.605), (0.30, 0.585), (0.20, 0.522), (0.01, 0.505)]),
    "eel": (
        [(0.05, 0.465), (0.18, 0.43), (0.44, 0.42), (0.68, 0.435), (0.81, 0.475)],
        [(0.81, 0.555), (0.68, 0.595), (0.44, 0.61), (0.20, 0.595), (0.05, 0.53)]),
    "round": (  # goldfish
        [(0.13, 0.49), (0.20, 0.35), (0.36, 0.27), (0.54, 0.29), (0.67, 0.38), (0.75, 0.46)],
        [(0.75, 0.57), (0.66, 0.71), (0.50, 0.795), (0.34, 0.775), (0.21, 0.65), (0.13, 0.545)]),
    "sturgeon": (
        [(0.02, 0.50), (0.08, 0.45), (0.20, 0.39), (0.44, 0.375), (0.66, 0.40), (0.78, 0.45)],
        [(0.78, 0.56), (0.64, 0.60), (0.42, 0.615), (0.24, 0.60), (0.10, 0.55), (0.02, 0.525)]),
    "sturgeon_short": (
        [(0.05, 0.495), (0.12, 0.44), (0.24, 0.385), (0.46, 0.37), (0.66, 0.40), (0.78, 0.45)],
        [(0.78, 0.56), (0.64, 0.605), (0.42, 0.62), (0.26, 0.605), (0.12, 0.55), (0.05, 0.52)]),
}
TAIL_ATTACH = {  # plan -> (x, top y, bottom y) of the peduncle
    "std": (0.77, 0.455, 0.565), "deep": (0.78, 0.455, 0.565),
    "long": (0.76, 0.445, 0.565), "duck": (0.76, 0.445, 0.565),
    "needle": (0.76, 0.45, 0.56), "eel": (0.80, 0.475, 0.555),
    "round": (0.74, 0.46, 0.57), "sturgeon": (0.77, 0.45, 0.56),
    "sturgeon_short": (0.77, 0.45, 0.56),
}
EYE = {  # plan -> (x, y)
    "std": (0.145, 0.455), "deep": (0.225, 0.42), "long": (0.155, 0.455),
    "duck": (0.225, 0.45), "needle": (0.265, 0.46), "eel": (0.125, 0.465),
    "round": (0.235, 0.44), "sturgeon": (0.135, 0.465), "sturgeon_short": (0.165, 0.46),
}
GILL = {"std": 0.225, "deep": 0.315, "long": 0.235, "duck": 0.30,
        "needle": 0.335, "eel": 0.21, "round": 0.315,
        "sturgeon": 0.21, "sturgeon_short": 0.24}

def tail_points(kind, plan):
    ax, tt, tb = TAIL_ATTACH[plan]
    cy = (tt + tb) / 2.0
    if kind == "forked":
        return [(ax, tt), (0.985, tt - 0.135), (0.90, cy), (0.985, tb + 0.135), (ax, tb)]
    if kind == "flow":  # goldfish: long flowing fork
        return [(ax, tt), (0.995, tt - 0.17), (0.915, cy), (0.995, tb + 0.17), (ax, tb)]
    if kind == "crescent":
        return [(ax, tt), (0.99, tt - 0.20), (0.895, cy), (0.99, tb + 0.20), (ax, tb)]
    if kind == "rounded":
        return [(ax, tt), (0.925, tt - 0.035), (0.975, cy), (0.925, tb + 0.035), (ax, tb)]
    if kind == "square":
        return [(ax, tt), (0.96, tt - 0.05), (0.965, cy), (0.96, tb + 0.05), (ax, tb)]
    if kind == "hetero":  # sturgeon: long upper lobe
        return [(ax, tt), (0.99, tt - 0.165), (0.915, cy - 0.02), (0.95, tb + 0.075), (ax, tb)]
    raise ValueError(kind)

def fin_poly(d, pts, fill, size):
    """Polygon fin with a dark outline (drawn shrunk-fill over grown outline)."""
    S = lambda ps: [(x * size, y * size) for x, y in ps]
    cx = sum(x for x, _ in pts) / len(pts); cy = sum(y for _, y in pts) / len(pts)
    grown = [(cx + (x - cx) * 1.16, cy + (y - cy) * 1.16) for x, y in pts]
    d.polygon(S(grown), fill=OUTLINE)
    d.polygon(S(pts), fill=fill)

def dorsal_pts(kind, plan, top):
    ty = lambda x: interp(top, x)
    if kind == "single":
        return [(0.335, ty(0.335)), (0.40, ty(0.40) - 0.105), (0.545, ty(0.545) - 0.045), (0.575, ty(0.575))]
    if kind == "spiny":  # bass/perch/sunfish: tall spiny front, low soft rear
        pts = [(0.315, ty(0.315))]
        for i in range(5):
            x = 0.33 + i * 0.045
            pts.append((x, ty(x) - 0.115 + (i % 2) * 0.028))
        pts += [(0.56, ty(0.56) - 0.075), (0.635, ty(0.635) - 0.03), (0.65, ty(0.65))]
        return pts
    if kind == "long":  # bowfin: runs most of the back
        return [(0.315, ty(0.315)), (0.36, ty(0.36) - 0.055), (0.55, ty(0.55) - 0.05),
                (0.72, ty(0.72) - 0.045), (0.775, ty(0.775))]
    if kind == "longlow":  # carp: long, low
        return [(0.30, ty(0.30)), (0.35, ty(0.35) - 0.07), (0.55, ty(0.55) - 0.055), (0.63, ty(0.63))]
    if kind == "rear":  # pike family: set far back, anal fin mirrors it
        return [(0.615, ty(0.615)), (0.66, ty(0.66) - 0.085), (0.755, ty(0.755) - 0.03), (0.775, ty(0.775))]
    if kind == "eelback":  # burbot: long second dorsal
        return [(0.36, ty(0.36)), (0.42, ty(0.42) - 0.05), (0.62, ty(0.62) - 0.045), (0.80, ty(0.80))]
    return None

def draw_fish(size, cfg):
    plan = cfg.get("plan", "std")
    top, bot = PLANS[plan]
    if cfg.get("hscale"):
        cy0 = 0.515
        top = [(x, cy0 + (y - cy0) * cfg["hscale"]) for x, y in top]
        bot = [(x, cy0 + (y - cy0) * cfg["hscale"]) for x, y in bot]
    poly = top + bot
    back, flank, belly = cfg["back"], cfg["flank"], cfg["belly"]
    hi = shade(flank, 1.28)
    fin = cfg.get("fin", shade(flank, 0.82))
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    S = lambda ps: [(x * size, y * size) for x, y in ps]
    rng = __import__("random").Random(zlib.crc32(cfg["id"].encode()))

    # --- tail + rear fins (behind body)
    tp = tail_points(cfg.get("tail", "forked"), plan)
    fin_poly(d, tp, fin, size)
    if cfg.get("tail_white_tip"):
        d.polygon(S([(0.90, 0.60), (0.985, 0.70), (0.90, 0.635)]), fill=(245, 245, 238, 255))
    dp = dorsal_pts(cfg.get("dorsal", "single"), plan, top)
    if dp:
        fin_poly(d, dp, fin, size)
    if cfg.get("dorsal") == "triple":
        for cx in (0.315, 0.475, 0.615):
            ty = lambda x: interp(top, x)
            fin_poly(d, [(cx - 0.055, ty(cx - 0.055)), (cx, ty(cx) - 0.085),
                         (cx + 0.06, ty(cx + 0.06) - 0.02), (cx + 0.075, ty(cx + 0.075))], fin, size)
    if cfg.get("dorsal") == "rear":  # anal fin mirrors the rear dorsal
        by = lambda x: interp(bot, x)
        fin_poly(d, [(0.615, by(0.615)), (0.66, by(0.66) + 0.085),
                     (0.755, by(0.755) + 0.03), (0.775, by(0.775))], fin, size)
    if cfg.get("dorsal") == "eelback":
        by = lambda x: interp(bot, x)
        fin_poly(d, [(0.36, by(0.36)), (0.42, by(0.42) + 0.05),
                     (0.62, by(0.62) + 0.045), (0.80, by(0.80))], fin, size)
    if cfg.get("adipose"):
        ty = lambda x: interp(top, x)
        fin_poly(d, [(0.665, ty(0.665)), (0.695, ty(0.695) - 0.038), (0.735, ty(0.735))],
                 shade(fin, 0.9), size)
    if cfg.get("finlets"):
        ty = lambda x: interp(top, x); by = lambda x: interp(bot, x)
        for fx in (0.665, 0.715, 0.765):
            fin_poly(d, [(fx, ty(fx)), (fx + 0.018, ty(fx) - 0.035), (fx + 0.038, ty(fx))], fin, size)
            fin_poly(d, [(fx, by(fx)), (fx + 0.018, by(fx) + 0.035), (fx + 0.038, by(fx))], fin, size)
    # anal + pelvic fins (behind body)
    by = lambda x: interp(bot, x)
    if plan not in ("eel",) and cfg.get("dorsal") != "rear":
        fin_poly(d, [(0.575, by(0.575)), (0.625, by(0.625) + 0.075), (0.70, by(0.70) + 0.015),
                     (0.715, by(0.715))], fin, size)
    fin_poly(d, [(0.315, by(0.315)), (0.355, by(0.355) + 0.065), (0.415, by(0.415))], fin, size)

    # --- body: column fill with back/flank/belly gradient + highlight band
    px = img.load()
    cols = {}
    for xi in range(size):
        x = (xi + 0.5) / size
        ys = []
        n = len(poly)
        for i in range(n):
            x0, y0 = poly[i]; x1, y1 = poly[(i + 1) % n]
            if (x0 <= x < x1) or (x1 <= x < x0):
                ys.append(y0 + (y1 - y0) * (x - x0) / (x1 - x0))
        if len(ys) < 2:
            continue
        ymin, ymax = min(ys), max(ys)
        y0i, y1i = int(ymin * size), int(ymax * size)
        cols[xi] = (y0i, y1i)
        for yi in range(y0i, y1i + 1):
            if not (0 <= yi < size):
                continue
            frac = (yi - y0i) / max(1, (y1i - y0i))
            if yi in (y0i, y1i):
                c = OUTLINE
            elif frac < 0.30:
                c = back
            elif frac < 0.46:
                c = hi
            elif frac < 0.68:
                c = flank
                if cfg.get("pinkband") and 0.44 < frac < 0.64:
                    c = cfg["pinkband"]
            else:
                c = belly
            if cfg.get("latline") and abs(frac - 0.52) < 0.045 and yi not in (y0i, y1i):
                c = cfg["latline"]
            px[xi, yi] = c

    inner = [(0.5 + (x - 0.5) * 0.93, 0.515 + (y - 0.515) * 0.90) for x, y in poly]

    pat = cfg.get("pattern")
    if pat == "spots":
        col = cfg.get("pat_color", shade(back, 0.55))
        n = cfg.get("pat_n", 46)
        for _ in range(n):
            xi = rng.randrange(2, size - 2)
            if xi not in cols:
                continue
            y0i, y1i = cols[xi]
            frac_hi = 1.0 if cfg.get("pat_all") else 0.62
            yi = rng.randrange(y0i + 1, max(y0i + 2, int(y0i + (y1i - y0i) * frac_hi)))
            if 0 <= yi < size and px[xi, yi] != OUTLINE:
                px[xi, yi] = col
                if rng.random() < 0.3 and xi + 1 < size:
                    px[xi + 1, yi] = col
        if cfg.get("pat_color2"):
            for _ in range(n // 3):
                xi = rng.randrange(2, size - 2)
                if xi not in cols:
                    continue
                y0i, y1i = cols[xi]
                yi = rng.randrange(y0i + 1, max(y0i + 2, int(y0i + (y1i - y0i) * 0.7)))
                if 0 <= yi < size and px[xi, yi] != OUTLINE:
                    px[xi, yi] = cfg["pat_color2"]
    elif pat == "lightspots":
        col = cfg.get("pat_color", (238, 232, 200, 255))
        for _ in range(cfg.get("pat_n", 40)):
            xi = rng.randrange(2, size - 2)
            if xi not in cols:
                continue
            y0i, y1i = cols[xi]
            yi = rng.randrange(y0i + 1, max(y0i + 2, y1i - 1))
            if 0 <= yi < size and px[xi, yi] != OUTLINE:
                px[xi, yi] = col
    elif pat == "bars":
        col = cfg.get("pat_color", shade(back, 0.6))
        for bx, bw in cfg["bars"]:
            for xi in range(size):
                x = (xi + 0.5) / size
                if abs(x - bx) < bw and xi in cols:
                    y0i, y1i = cols[xi]
                    for yi in range(y0i + 1, y1i):
                        if px[xi, yi] != OUTLINE:
                            px[xi, yi] = col
    elif pat == "stripes":
        col = cfg.get("pat_color", shade(back, 0.55))
        for fr in (0.24, 0.335, 0.43, 0.525, 0.62, 0.715, 0.81):
            for xi in range(size):
                if xi not in cols:
                    continue
                y0i, y1i = cols[xi]
                yi = int(y0i + fr * (y1i - y0i))
                if y0i < yi < y1i and px[xi, yi] != OUTLINE:
                    px[xi, yi] = col
    elif pat == "mottle":
        col = cfg.get("pat_color", shade(back, 0.6))
        ov = Image.new("RGBA", (size, size), (0, 0, 0, 0))
        od = ImageDraw.Draw(ov)
        for _ in range(cfg.get("pat_n", 16)):
            cx, cy = rng.uniform(0.08, 0.76), rng.uniform(0.28, 0.78)
            r = rng.uniform(0.03, 0.075)
            od.ellipse([(cx - r) * size, (cy - r * 0.7) * size, (cx + r) * size, (cy + r * 0.7) * size], fill=col)
        mask = Image.new("L", (size, size), 0)
        ImageDraw.Draw(mask).polygon(S(inner), fill=255)
        img.paste(ov, (0, 0), _mask_alpha(ov, mask))
        px = img.load()
    elif pat == "chain":
        col = cfg.get("pat_color", shade(back, 0.55))
        ov = Image.new("RGBA", (size, size), (0, 0, 0, 0))
        od = ImageDraw.Draw(ov)
        for k in range(-8, 16):
            od.line([(k * 4, 0), (k * 4 + size, size)], fill=col, width=1)
            od.line([(k * 4, size), (k * 4 + size, 0)], fill=col, width=1)
        mask = Image.new("L", (size, size), 0)
        ImageDraw.Draw(mask).polygon(S(inner), fill=255)
        img.paste(ov, (0, 0), _mask_alpha(ov, mask))
        px = img.load()
    elif pat == "wavy":  # mackerel back markings
        col = cfg.get("pat_color", shade(back, 0.55))
        for xi in range(size):
            if xi not in cols:
                continue
            y0i, y1i = cols[xi]
            for base_fr in (0.10, 0.24, 0.38):
                fr = base_fr + 0.05 * math.sin(xi * 0.9 + base_fr * 9.0)
                yi = int(y0i + fr * (y1i - y0i))
                if y0i < yi < y1i and px[xi, yi] != OUTLINE:
                    px[xi, yi] = col
    if cfg.get("scales"):
        ov = Image.new("RGBA", (size, size), (0, 0, 0, 0))
        od = ImageDraw.Draw(ov)
        sc = shade(flank, 0.78)
        for row in range(4):
            for k in range(7):
                cx = (0.16 + k * 0.085 + (row % 2) * 0.042) * size
                cy = (0.38 + row * 0.085) * size
                od.arc([cx - 3, cy - 3, cx + 3, cy + 3], start=300, end=120, fill=sc, width=1)
        mask = Image.new("L", (size, size), 0)
        ImageDraw.Draw(mask).polygon(S(inner), fill=255)
        img.paste(ov, (0, 0), _mask_alpha(ov, mask))
        px = img.load()

    # --- head details
    gx = GILL[plan]
    gy0, gy1 = interp(top, gx), interp(bot, gx)
    d.line([(gx + 0.012) * size, (gy0 + 0.02) * size, (gx - 0.008) * size, (gy1 - 0.02) * size],
           fill=shade(flank, 0.62), width=max(1, size // 32))
    if cfg.get("earflap"):
        ex, ey = (gx - 0.015) * size, (gy0 + 0.055) * size
        r = size * 0.045
        d.ellipse([ex - r, ey - r * 0.8, ex + r, ey + r * 0.8], fill=cfg["earflap"])
    if cfg.get("cheek"):
        cxp, cyp = EYE[plan][0] * size, (EYE[plan][1] + 0.115) * size
        r = size * 0.055
        d.ellipse([cxp - r, cyp - r * 0.7, cxp + r, cyp + r * 0.7], fill=cfg["cheek"])
    if cfg.get("blotch"):  # haddock shoulder blotch
        bx, byp = 0.315 * size, 0.435 * size
        r = size * 0.05
        d.ellipse([bx - r, byp - r, bx + r, byp + r], fill=cfg["blotch"])
    ex, ey = EYE[plan]
    er = size * (0.052 if cfg.get("bigeye") else 0.042)
    exc, eyc = ex * size, ey * size
    d.ellipse([exc - er, eyc - er, exc + er, eyc + er], fill=(235, 232, 210, 255))
    pr = er * (0.62 if not cfg.get("redeye") else 0.66)
    d.ellipse([exc - pr, eyc - pr, exc + pr, eyc + pr],
              fill=(178, 40, 26, 255) if cfg.get("redeye") else (18, 14, 10, 255))
    # mouth
    mx, my = top[0][0], (top[0][1] + bot[-1][1]) / 2.0
    if cfg.get("mouth") == "big":
        d.line([mx * size, my * size, (mx + 0.155) * size, (my - 0.028) * size],
               fill=OUTLINE, width=max(1, size // 32))
        d.line([(mx + 0.02) * size, (my + 0.045) * size, (mx + 0.13) * size, (my + 0.03) * size],
               fill=OUTLINE, width=1)
    elif cfg.get("mouth") == "down":  # sucker: underslung round mouth
        d.arc([(mx + 0.005) * size, (my + 0.028) * size, (mx + 0.075) * size, (my + 0.085) * size],
              start=200, end=340, fill=OUTLINE, width=max(1, size // 32))
    elif cfg.get("mouth") == "under":  # sturgeon: under the snout + barbels
        d.line([(mx + 0.055) * size, (my + 0.055) * size, (mx + 0.115) * size, (my + 0.05) * size],
               fill=OUTLINE, width=max(1, size // 32))
        for bx in (0.075, 0.095, 0.115, 0.135):
            byy = interp(bot, bx)
            d.line([bx * size, byy * size, (bx - 0.008) * size, (byy + 0.045) * size],
                   fill=OUTLINE, width=1)
    elif plan in ("duck", "needle"):
        d.line([mx * size, (my - 0.004) * size, (mx + 0.145) * size, (my - 0.004) * size],
               fill=OUTLINE, width=max(1, size // 32))
        if plan == "needle":  # gar teeth ticks
            for tx in (0.05, 0.09, 0.13, 0.17):
                d.line([tx * size, (my + 0.012) * size, tx * size, (my + 0.032) * size],
                       fill=(240, 238, 228, 255), width=1)
    else:
        d.line([mx * size, my * size, (mx + 0.062) * size, (my - 0.016) * size],
               fill=OUTLINE, width=max(1, size // 32))
    if cfg.get("teeth"):  # bluefish: small white fangs on the jaw line
        for tx in (0.075, 0.105, 0.135):
            d.line([tx * size, (my + 0.018) * size, (tx + 0.006) * size, (my + 0.045) * size],
                   fill=(242, 240, 232, 255), width=1)
    if cfg.get("barbels"):  # catfish whiskers from the mouth corner
        bx0, by0 = (mx + 0.045) * size, (my + 0.01) * size
        for ln, dy in ((0.16, 0.10), (0.13, 0.155), (0.09, 0.05)):
            d.line([bx0, by0, (bx0 / size + ln) * size, by0 + dy * size],
                   fill=OUTLINE, width=max(1, size // 32))
        d.line([bx0, by0 + 2, bx0 + 0.05 * size, by0 + 0.075 * size], fill=OUTLINE, width=1)
    if cfg.get("chin_barbel"):
        cxp = (mx + 0.075) * size
        cyp = interp(bot, mx + 0.075) * size
        d.line([cxp, cyp, cxp - 1, cyp + size * 0.05], fill=OUTLINE, width=max(1, size // 32))
    if cfg.get("mouth_barbel"):  # carp: one tiny barbel at the mouth corner
        d.line([(mx + 0.05) * size, (my + 0.03) * size, (mx + 0.085) * size, (my + 0.065) * size],
               fill=OUTLINE, width=1)
    # pectoral fin over the body, behind the gill
    pect = [(gx + 0.005, gy0 + 0.16 * (gy1 - gy0)), (gx + 0.095, gy0 + 0.30 * (gy1 - gy0)),
            (gx + 0.075, gy0 + 0.52 * (gy1 - gy0)), (gx + 0.005, gy0 + 0.42 * (gy1 - gy0))]
    fin_poly(d, pect, shade(fin, 0.92), size)
    if cfg.get("scutes"):  # sturgeon armor: pale bony plates in rows
        plate = (206, 196, 168, 255)
        for sx in (0.30, 0.42, 0.54, 0.66):
            for fr, rr in ((0.50, 0.030), (0.16, 0.020)):
                cxp = sx * size
                cyp = (interp(top, sx) + fr * (interp(bot, sx) - interp(top, sx))) * size
                r = rr * size
                d.polygon([(cxp - r, cyp), (cxp, cyp - r), (cxp + r, cyp), (cxp, cyp + r)], fill=plate)
    return img

def _mask_alpha(ov, mask):
    """Alpha of overlay clipped by an L mask (patterns stay inside the body)."""
    a = ov.split()[3]
    return Image.composite(a, Image.new("L", ov.size, 0), mask)

# ---------------------------------------------------------------- flatfish
def draw_flatfish(size, cfg):
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    S = lambda ps: [(x * size, y * size) for x, y in ps]
    rng = __import__("random").Random(zlib.crc32(cfg["id"].encode()))
    base, finc = cfg["flank"], cfg.get("fin", (196, 172, 138, 255))
    body = [(0.07, 0.50), (0.22, 0.27), (0.46, 0.185), (0.70, 0.25), (0.86, 0.40),
            (0.90, 0.50), (0.86, 0.60), (0.70, 0.75), (0.46, 0.815), (0.22, 0.73)]
    fringe = [(0.5 + (x - 0.5) * 1.09, 0.5 + (y - 0.5) * 1.13) for x, y in body]
    d.polygon(S(fringe), fill=finc)                       # fin fringe
    d.polygon(S([(0.5 + (x - 0.5) * 1.045, 0.5 + (y - 0.5) * 1.06) for x, y in body]), fill=OUTLINE)
    d.polygon(S(body), fill=base)
    inner = [(0.5 + (x - 0.5) * 0.92, 0.5 + (y - 0.5) * 0.92) for x, y in body]
    ov = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    od = ImageDraw.Draw(ov)
    for _ in range(18):  # camouflage mottle + pale speckles
        cx, cy = rng.uniform(0.14, 0.84), rng.uniform(0.26, 0.74)
        r = rng.uniform(0.025, 0.06)
        od.ellipse([(cx - r) * size, (cy - r * 0.7) * size, (cx + r) * size, (cy + r * 0.7) * size],
                   fill=shade(base, 0.66))
    for _ in range(22):
        cx, cy = rng.uniform(0.14, 0.84), rng.uniform(0.26, 0.74)
        od.point((cx * size, cy * size), fill=(226, 210, 178, 255))
    mask = Image.new("L", (size, size), 0)
    ImageDraw.Draw(mask).polygon(S(inner), fill=255)
    img.paste(ov, (0, 0), _mask_alpha(ov, mask))
    # tail fan at the right tip
    fin_poly(d, [(0.885, 0.44), (0.975, 0.40), (0.985, 0.50), (0.975, 0.60), (0.885, 0.56)], finc, size)
    # both eyes on the eyed (upper) side, near the head
    for ex, ey, r in ((0.205, 0.415, 0.030), (0.275, 0.365, 0.026)):
        exc, eyc = ex * size, ey * size
        d.ellipse([exc - r * size, eyc - r * size, exc + r * size, eyc + r * size], fill=(238, 234, 214, 255))
        pr = r * size * 0.6
        d.ellipse([exc - pr, eyc - pr, exc + pr, eyc + pr], fill=(16, 12, 8, 255))
    d.arc([0.045 * size, 0.47 * size, 0.11 * size, 0.55 * size], start=250, end=60,
          fill=OUTLINE, width=max(1, size // 32))
    return img

# ---------------------------------------------------------------- crayfish
def draw_crayfish(size, cfg):
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    S = lambda ps: [(x * size, y * size) for x, y in ps]
    body = cfg["flank"]; dark = shade(body, 0.55); light = shade(body, 1.3)
    mega = cfg.get("mega", False)
    # long antennae sweeping forward from the head
    for sgn, reach in ((1, 0.30), (-1, 0.22)):
        pts = [(0.30, 0.44), (0.20, 0.44 - sgn * 0.10), (0.10, 0.44 - sgn * reach),
               (0.03, 0.44 - sgn * (reach + 0.10))]
        d.line(S(pts), fill=dark, width=max(1, size // 32))
    # walking legs under the thorax
    for lx in (0.36, 0.44, 0.52):
        d.line(S([(lx, 0.60), (lx - 0.03, 0.74), (lx + 0.015, 0.85)]), fill=dark, width=max(1, size // 32))
    # segmented abdomen curling back to a tail fan
    segs = [(0.60, 0.475, 0.105), (0.695, 0.49, 0.095), (0.775, 0.51, 0.082),
            (0.84, 0.535, 0.068), (0.885, 0.56, 0.054)]
    for cx, cy, r in reversed(segs):
        d.ellipse([(cx - r) * size, (cy - r * 0.78) * size, (cx + r) * size, (cy + r * 0.78) * size], fill=dark)
        ri = r * 0.84
        d.ellipse([(cx - ri) * size, (cy - ri * 0.78) * size, (cx + ri) * size, (cy + ri * 0.78) * size], fill=body)
        d.arc([(cx - ri) * size, (cy - ri * 0.78) * size, (cx + ri) * size, (cy + ri * 0.78) * size],
              start=300, end=60, fill=light, width=max(1, size // 32))
    # tail fan
    fin_poly(d, [(0.90, 0.52), (0.985, 0.44), (0.99, 0.56), (0.975, 0.68), (0.895, 0.615)],
             shade(body, 0.85), size)
    # cephalothorax shield + rostrum spike
    shield = [(0.155, 0.475), (0.24, 0.345), (0.40, 0.315), (0.545, 0.36), (0.585, 0.475),
              (0.545, 0.60), (0.40, 0.645), (0.24, 0.615)]
    d.polygon(S([(0.37 + (x - 0.37) * 1.07, 0.48 + (y - 0.48) * 1.09) for x, y in shield]), fill=dark)
    d.polygon(S(shield), fill=body)
    d.polygon(S([(0.16, 0.445), (0.045, 0.475), (0.16, 0.515)]), fill=dark)  # rostrum
    d.ellipse([0.245 * size, 0.365 * size, 0.475 * size, 0.50 * size], fill=light)  # highlight
    d.arc([0.30 * size, 0.33 * size, 0.52 * size, 0.63 * size], start=290, end=70,
          fill=dark, width=max(1, size // 32))  # cervical groove
    # eye stalks
    for ex in (0.205, 0.245):
        d.line(S([(ex, 0.42), (ex - 0.012, 0.375)]), fill=dark, width=max(1, size // 32))
        r = size * 0.020
        d.ellipse([(ex - 0.012) * size - r, 0.375 * size - r, (ex - 0.012) * size + r, 0.375 * size + r],
                  fill=(14, 12, 10, 255))
    # pincer claws in front (one high, one low, offset pair)
    def claw(cx, cy, s, tone):
        palm = [(cx - 0.10 * s, cy), (cx - 0.03 * s, cy - 0.075 * s), (cx + 0.075 * s, cy - 0.045 * s),
                (cx + 0.10 * s, cy + 0.02 * s), (cx + 0.04 * s, cy + 0.07 * s), (cx - 0.05 * s, cy + 0.065 * s)]
        d.polygon(S([(cx + (x - cx) * 1.14, cy + (y - cy) * 1.14) for x, y in palm]), fill=dark)
        d.polygon(S(palm), fill=tone)
        d.polygon(S([(cx - 0.115 * s, cy - 0.012 * s), (cx - 0.20 * s, cy - 0.075 * s),
                     (cx - 0.085 * s, cy + 0.028 * s)]), fill=tone)   # fixed finger
        d.polygon(S([(cx - 0.10 * s, cy + 0.045 * s), (cx - 0.185 * s, cy + 0.10 * s),
                     (cx - 0.065 * s, cy + 0.055 * s)]), fill=shade(tone, 0.9))  # movable finger
    claw(0.185, 0.635, 1.15 if mega else 0.8, body)
    claw(0.265, 0.72, 0.9 if mega else 0.62, shade(body, 0.82))
    return img

# ---------------------------------------------------------------- fry
def draw_fry(size):
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    S = lambda ps: [(x * size, y * size) for x, y in ps]
    pale = (198, 214, 218, 255); paler = (226, 236, 238, 255)
    # tiny larva centered: yolk sac, thread body, big eye, finfold tail
    fin_poly(d, [(0.60, 0.475), (0.71, 0.435), (0.665, 0.50), (0.71, 0.565), (0.60, 0.525)],
             paler, size)
    body = [(0.315, 0.50), (0.36, 0.455), (0.48, 0.445), (0.575, 0.465), (0.615, 0.50),
            (0.575, 0.535), (0.48, 0.555), (0.36, 0.545)]
    d.polygon(S([(0.465 + (x - 0.465) * 1.10, 0.5 + (y - 0.5) * 1.22) for x, y in body]),
              fill=shade(pale, 0.8))
    d.polygon(S(body), fill=pale)
    d.ellipse([0.355 * size, 0.515 * size, 0.475 * size, 0.60 * size], fill=(238, 204, 152, 255))  # yolk sac
    er = size * 0.052
    exc, eyc = 0.375 * size, 0.485 * size
    d.ellipse([exc - er, eyc - er, exc + er, eyc + er], fill=(240, 244, 246, 255))
    pr = er * 0.66
    d.ellipse([exc - pr, eyc - pr, exc + pr, eyc + pr], fill=(20, 24, 26, 255))
    return img

# ---------------------------------------------------------------- species
# back / flank / belly / fin + plan, tail, dorsal, pattern & extras per id.
SILVER = (238, 240, 238, 255)
SP = [
    ("mon_crayfish_giant_larva", dict(kind="crayfish", flank=(188, 172, 158, 255))),
    ("mon_crayfish_mega", dict(kind="crayfish", mega=True, flank=(128, 126, 120, 255))),
    ("mon_fish_blackfish", dict(back=(52, 56, 58, 255), flank=(86, 90, 90, 255), belly=(168, 170, 164, 255),
                                dorsal="spiny", tail="forked", pattern="mottle",
                                pat_color=(150, 152, 146, 255), pat_n=12)),
    ("mon_fish_bluefin_tuna", dict(back=(28, 48, 108, 255), flank=(96, 128, 178, 255), belly=SILVER,
                                   tail="crescent", dorsal="single", finlets=True, hscale=0.88,
                                   fin=(36, 58, 118, 255))),
    ("mon_fish_bluefish", dict(back=(36, 104, 106, 255), flank=(110, 168, 168, 255), belly=SILVER,
                               tail="forked", dorsal="single", teeth=True)),
    ("mon_fish_bluegill", dict(plan="deep", back=(84, 96, 52, 255), flank=(140, 152, 92, 255),
                               belly=(226, 206, 150, 255), dorsal="spiny", tail="forked",
                               pattern="bars", pat_color=(96, 108, 62, 255),
                               bars=[(0.42, 0.028), (0.54, 0.028), (0.66, 0.028)],
                               earflap=(30, 30, 26, 255), cheek=(86, 130, 168, 255))),
    ("mon_fish_bowfin", dict(plan="long", back=(88, 92, 52, 255), flank=(128, 130, 84, 255),
                             belly=(206, 200, 160, 255), dorsal="long", tail="rounded")),
    ("mon_fish_brook_trout", dict(back=(92, 96, 56, 255), flank=(140, 138, 88, 255), belly=(232, 224, 196, 255),
                                  tail="square", dorsal="single", adipose=True, pattern="lightspots",
                                  pat_color=(238, 230, 188, 255), pat_n=30, pat_color2=(196, 84, 52, 255))),
    ("mon_fish_brown_trout", dict(back=(110, 88, 44, 255), flank=(168, 140, 82, 255), belly=(238, 228, 196, 255),
                                  tail="forked", dorsal="single", adipose=True, pattern="spots",
                                  pat_color=(52, 40, 26, 255), pat_n=34, pat_all=True,
                                  pat_color2=(188, 70, 44, 255))),
    ("mon_fish_bullhead", dict(back=(92, 68, 44, 255), flank=(128, 100, 66, 255), belly=(216, 198, 168, 255),
                               tail="rounded", dorsal="spiny", barbels=True)),
    ("mon_fish_bullhead_yellow", dict(back=(140, 112, 52, 255), flank=(186, 156, 84, 255),
                                      belly=(240, 228, 188, 255), tail="rounded", dorsal="spiny",
                                      barbels=True)),
    ("mon_fish_burbot", dict(plan="eel", back=(122, 102, 66, 255), flank=(164, 142, 100, 255),
                             belly=(228, 216, 188, 255), dorsal="eelback", tail="rounded",
                             pattern="mottle", pat_color=(96, 78, 50, 255), pat_n=12,
                             chin_barbel=True)),
    ("mon_fish_calico_bass", dict(plan="deep", back=(96, 108, 84, 255), flank=(168, 176, 152, 255),
                                  belly=(236, 238, 226, 255), dorsal="spiny", tail="forked",
                                  pattern="mottle", pat_color=(40, 44, 36, 255), pat_n=18)),
    ("mon_fish_carp", dict(back=(128, 110, 44, 255), flank=(186, 158, 72, 255), belly=(240, 226, 180, 255),
                           dorsal="longlow", tail="forked", scales=True, mouth_barbel=True)),
    ("mon_fish_channel_catfish", dict(back=(84, 100, 122, 255), flank=(128, 144, 164, 255),
                                      belly=(232, 236, 238, 255), tail="forked", dorsal="spiny",
                                      barbels=True, pattern="spots", pat_color=(60, 72, 92, 255),
                                      pat_n=12, pat_all=True)),
    ("mon_fish_cod", dict(back=(104, 108, 74, 255), flank=(156, 156, 120, 255), belly=(238, 236, 222, 255),
                          dorsal="triple", tail="forked", chin_barbel=True, pattern="mottle",
                          pat_color=(196, 196, 168, 255), pat_n=10,
                          latline=(232, 230, 210, 255))),
    ("mon_fish_fallfish", dict(back=(118, 124, 92, 255), flank=(182, 186, 164, 255), belly=SILVER,
                               tail="forked", dorsal="single")),
    ("mon_fish_flounder", dict(kind="flat", flank=(128, 102, 70, 255))),
    ("mon_fish_gar_longnose", dict(plan="needle", back=(96, 102, 62, 255), flank=(146, 150, 104, 255),
                                   belly=(226, 224, 196, 255), dorsal="rear", tail="rounded",
                                   pattern="spots", pat_color=(56, 60, 38, 255), pat_n=20, pat_all=True)),
    ("mon_fish_goldfish", dict(plan="round", back=(238, 128, 26, 255), flank=(250, 156, 44, 255),
                               belly=(255, 196, 104, 255), dorsal="single", tail="flow",
                               fin=(255, 178, 92, 255))),
    ("mon_fish_grass_carp", dict(back=(120, 116, 62, 255), flank=(178, 168, 104, 255),
                                 belly=(238, 232, 204, 255), dorsal="single", tail="forked",
                                 scales=True)),
    ("mon_fish_green_sunfish", dict(plan="deep", back=(70, 100, 58, 255), flank=(116, 152, 84, 255),
                                    belly=(232, 214, 158, 255), dorsal="spiny", tail="forked",
                                    mouth="big", earflap=(28, 30, 24, 255), pattern="spots",
                                    pat_color=(92, 140, 168, 255), pat_n=16, pat_all=True,
                                    fin=(196, 130, 62, 255))),
    ("mon_fish_haddock", dict(back=(112, 100, 92, 255), flank=(172, 162, 152, 255), belly=SILVER,
                              dorsal="triple", tail="forked", chin_barbel=True,
                              blotch=(26, 24, 22, 255), latline=(40, 36, 32, 255))),
    ("mon_fish_halibut", dict(kind="flat", flank=(106, 88, 64, 255))),
    ("mon_fish_kokanee_salmon", dict(back=(52, 84, 120, 255), flank=(148, 178, 200, 255), belly=SILVER,
                                     tail="forked", dorsal="single", adipose=True, pattern="spots",
                                     pat_color=(40, 58, 84, 255), pat_n=20)),
    ("mon_fish_lake_trout", dict(back=(84, 96, 84, 255), flank=(128, 142, 124, 255),
                                 belly=(226, 228, 210, 255), tail="forked", dorsal="single",
                                 adipose=True, pattern="lightspots", pat_n=42)),
    ("mon_fish_lbass", dict(back=(66, 100, 56, 255), flank=(124, 158, 96, 255), belly=(238, 240, 220, 255),
                            dorsal="spiny", tail="forked", mouth="big",
                            latline=(44, 66, 38, 255))),
    ("mon_fish_mackerel", dict(back=(34, 92, 110, 255), flank=(120, 168, 178, 255), belly=SILVER,
                               tail="crescent", dorsal="single", finlets=True, hscale=0.85,
                               pattern="wavy", pat_color=(24, 60, 76, 255))),
    ("mon_fish_muskellunge", dict(plan="duck", back=(104, 116, 76, 255), flank=(158, 168, 122, 255),
                                  belly=(236, 236, 214, 255), dorsal="rear", tail="forked",
                                  pattern="bars", pat_color=(80, 88, 56, 255),
                                  bars=[(0.40, 0.020), (0.50, 0.020), (0.60, 0.020), (0.70, 0.020)])),
    ("mon_fish_pbass", dict(back=(96, 116, 124, 255), flank=(178, 190, 192, 255), belly=SILVER,
                            dorsal="spiny", tail="forked", pattern="stripes",
                            pat_color=(52, 66, 72, 255))),
    ("mon_fish_perch", dict(back=(110, 116, 58, 255), flank=(196, 168, 76, 255), belly=(240, 230, 196, 255),
                            dorsal="spiny", tail="forked", pattern="bars",
                            pat_color=(70, 76, 44, 255),
                            bars=[(0.32, 0.026), (0.44, 0.026), (0.56, 0.026), (0.68, 0.026), (0.78, 0.022)],
                            fin=(226, 120, 52, 255))),
    ("mon_fish_pickerel", dict(plan="duck", back=(76, 96, 52, 255), flank=(140, 156, 88, 255),
                               belly=(232, 230, 198, 255), dorsal="rear", tail="forked",
                               pattern="chain", pat_color=(52, 66, 36, 255))),
    ("mon_fish_pickerel_american", dict(plan="duck", back=(84, 100, 54, 255), flank=(150, 160, 92, 255),
                                        belly=(234, 230, 200, 255), dorsal="rear", tail="forked",
                                        pattern="chain", pat_color=(60, 72, 40, 255),
                                        fin=(196, 110, 70, 255))),
    ("mon_fish_pike", dict(plan="duck", back=(84, 100, 58, 255), flank=(132, 150, 92, 255),
                           belly=(234, 234, 208, 255), dorsal="rear", tail="forked",
                           pattern="lightspots", pat_color=(226, 228, 188, 255), pat_n=34)),
    ("mon_fish_pollock", dict(back=(92, 104, 72, 255), flank=(158, 164, 132, 255), belly=SILVER,
                              dorsal="triple", tail="forked", latline=(230, 228, 205, 255))),
    ("mon_fish_porgy", dict(plan="deep", back=(108, 116, 140, 255), flank=(178, 182, 200, 255),
                            belly=SILVER, dorsal="spiny", tail="forked")),
    ("mon_fish_pumpkinseed", dict(plan="deep", back=(96, 110, 60, 255), flank=(170, 160, 84, 255),
                                  belly=(246, 190, 96, 255), dorsal="spiny", tail="forked",
                                  pattern="bars", pat_color=(110, 118, 70, 255),
                                  bars=[(0.44, 0.024), (0.58, 0.024)],
                                  earflap=(240, 110, 40, 255), cheek=(92, 148, 178, 255))),
    ("mon_fish_rainbow_trout", dict(back=(88, 108, 96, 255), flank=(176, 186, 176, 255), belly=SILVER,
                                    tail="forked", dorsal="single", adipose=True, pattern="spots",
                                    pat_color=(30, 34, 30, 255), pat_n=52, pat_all=True,
                                    pinkband=(228, 148, 158, 255))),
    ("mon_fish_redbreast", dict(plan="deep", back=(92, 104, 60, 255), flank=(146, 156, 96, 255),
                                belly=(238, 148, 84, 255), dorsal="spiny", tail="forked",
                                earflap=(26, 28, 24, 255), pattern="spots",
                                pat_color=(96, 160, 170, 255), pat_n=14, pat_all=True)),
    ("mon_fish_rock_bass", dict(plan="deep", back=(96, 102, 62, 255), flank=(148, 152, 104, 255),
                                belly=(228, 220, 184, 255), dorsal="spiny", tail="forked",
                                pattern="mottle", pat_color=(66, 72, 44, 255), pat_n=14,
                                redeye=True)),
    ("mon_fish_salmon", dict(back=(70, 96, 118, 255), flank=(168, 186, 198, 255), belly=SILVER,
                             tail="forked", dorsal="single", adipose=True, pattern="spots",
                             pat_color=(34, 44, 56, 255), pat_n=22)),
    ("mon_fish_sbass", dict(back=(110, 88, 56, 255), flank=(164, 136, 92, 255), belly=(238, 230, 204, 255),
                            dorsal="spiny", tail="forked", mouth="big", pattern="bars",
                            pat_color=(120, 96, 62, 255),
                            bars=[(0.36, 0.020), (0.47, 0.020), (0.58, 0.020), (0.69, 0.020)])),
    ("mon_fish_sturgeon_lake", dict(plan="sturgeon", back=(104, 96, 82, 255), flank=(142, 132, 114, 255),
                                    belly=(224, 216, 196, 255), dorsal="rear", tail="hetero",
                                    mouth="under", scutes=True, fin=(110, 102, 88, 255))),
    ("mon_fish_sturgeon_shortnose", dict(plan="sturgeon_short", back=(110, 102, 88, 255),
                                         flank=(148, 138, 120, 255), belly=(228, 220, 200, 255),
                                         dorsal="rear", tail="hetero", mouth="under", scutes=True,
                                         fin=(116, 108, 94, 255))),
    ("mon_fish_sunfish", dict(plan="deep", back=(88, 100, 58, 255), flank=(140, 152, 92, 255),
                              belly=(230, 216, 164, 255), dorsal="spiny", tail="forked",
                              pattern="bars", pat_color=(62, 72, 42, 255),
                              bars=[(0.40, 0.030), (0.53, 0.030), (0.66, 0.030)],
                              earflap=(28, 30, 24, 255))),
    ("mon_fish_walleye", dict(back=(96, 102, 58, 255), flank=(150, 152, 100, 255),
                              belly=(240, 240, 230, 255), dorsal="spiny", tail="forked",
                              bigeye=True, tail_white_tip=True)),
    ("mon_fish_warmouth", dict(plan="deep", back=(100, 88, 58, 255), flank=(152, 136, 96, 255),
                               belly=(230, 218, 188, 255), dorsal="spiny", tail="rounded",
                               mouth="big", pattern="mottle", pat_color=(70, 60, 40, 255),
                               pat_n=12, redeye=True, earflap=(30, 28, 22, 255))),
    ("mon_fish_white_catfish", dict(back=(148, 156, 164, 255), flank=(186, 192, 196, 255), belly=SILVER,
                                    tail="forked", dorsal="spiny", barbels=True)),
    ("mon_fish_white_sucker", dict(back=(108, 100, 68, 255), flank=(164, 152, 116, 255),
                                   belly=(238, 234, 214, 255), dorsal="single", tail="forked",
                                   mouth="down")),
    ("mon_fish_whitefish", dict(back=(104, 122, 132, 255), flank=(182, 194, 198, 255), belly=SILVER,
                                tail="forked", dorsal="single", adipose=True)),
    ("mon_fish_whitefish_round", dict(back=(110, 126, 134, 255), flank=(186, 196, 200, 255), belly=SILVER,
                                      tail="forked", dorsal="single", adipose=True)),
    ("mon_fry_blackfish", dict(kind="fry")),
]

CREDITS = ("Drawn from the monster's in-game description for the CDDA Voxel\n"
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.\n")

def render(mid, cfg):
    cfg = dict(cfg); cfg["id"] = mid
    kind = cfg.get("kind", "fish")
    if kind == "crayfish":
        return draw_crayfish(32, cfg)
    if kind == "flat":
        return draw_flatfish(32, cfg)
    if kind == "fry":
        return draw_fry(32)
    return draw_fish(32, cfg)

def main():
    jobs = []
    for mid, cfg in SP:
        rel = f"pngs_normal_32x32/monsters/{mid}/{mid}"
        folder = os.path.join(BASE, os.path.dirname(rel))
        os.makedirs(folder, exist_ok=True)
        img = render(mid, cfg)
        img.save(os.path.join(BASE, rel + ".png"))
        with open(os.path.join(BASE, rel + ".json"), "w") as f:
            json.dump([{"id": mid, "fg": mid, "bg": ""}], f, indent=2)
            f.write("\n")
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS)
        jobs.append((mid, img))
        print("wrote", rel + ".png", img.size)
    # contact-sheet preview: 8 columns, sprites at 2x over a neutral backdrop
    cols, cell = 8, 68
    rows = (len(jobs) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * cell, rows * cell), (44, 52, 60, 255))
    for i, (mid, img) in enumerate(jobs):
        big = img.resize((64, 64), Image.NEAREST)
        sheet.alpha_composite(big, ((i % cols) * cell + 2, (i // cols) * cell + 2))
    sheet.save(os.path.join(BASE, "preview_fish_a.png"))
    print("wrote preview_fish_a.png", sheet.size, f"({len(jobs)} sprites)")

if __name__ == "__main__":
    main()
