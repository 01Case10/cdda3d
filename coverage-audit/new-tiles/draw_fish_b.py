#!/usr/bin/env python3
"""
Draw the missing fish sprites, second half of the fish.json missing set,
from their game descriptions.

Sources (fork-src/cdda/data/json/monsters/fish.json, CDDA 6a3e59e0).
The second half is 52 ids: 50 fry plus the two adult mutant salmon.

  - 48 generic fish fry (mon_fry_*) all copy-from the mon_fry abstract:
    name "fish fry", symbol f, color white, volume 3 ml, description
    "A minuscule half-translucent fish larva.  Impossible to identify
    which species it belongs to."  The data intends them unidentifiable,
    so they share one larva design (fish-A's fry renderer) with small
    per-id seeded jitter in tint, yolk and proportions -- same creature,
    never a species portrait.  Translucency faked with pale opaque
    colors, per the no-semi-transparent-solids rule.
  - mon_fry_crayfish: own entry, name "crustacean larva", 3 ml,
    "A minuscule half-translucent crustacean larva." -- a zoea-style
    larva: stalked eyes, dorsal spine, segmented curl, tail fan, two
    minute claw buds.
  - mon_fry_mutant_salmon: 300 ml, "A half-translucent fish larva...
    though this one is far larger than the usual... it is going to
    outgrow normal fish" -- the same larva drawn much bigger in the
    tile, yolk tinted faintly pink toward the white_magenta adults.
  - mon_mutant_salmon_huge: 312 L, symbol S, color white_magenta,
    "A huge mutant salmon grown almost to the size of an alligator."
  - mon_mutant_salmon_mega ("leaper salmon"): 400 L, symbol S, color
    white_magenta, "...outgrows even the other ones present.  On its
    sides, beneath the fins, are visible two flesh-colored tubes which
    propel water as it leaps." -- drawn with the two flesh jet tubes.

Sheet placement by resolved volume (copy-from-resolved; standing
monster rule): under 100 L -> pngs_normal_32x32 (all 50 fry, 3 ml /
300 ml); 100-400 L -> pngs_large_64x64 (huge 312 L; mega exactly
400 L, which is not *over* 400, so it also lands on the large sheet,
drawn bulkier than huge within the same canvas); over 400 L ->
pngs_giant_96x96 (none in this half).

Design: profile view, head left, flat UltiCa style -- dark outline,
back/flank/belly gradient with one highlight band.  The fish renderer
is fish-A's, parameterized by canvas size so the 64x64 salmon are
drawn natively, not upscaled; the salmon keep the adult mon_fish_salmon
body plan (fish-A) washed toward magenta per their game color, bulked
up with hscale.  Output is UltiCa decomposed layout (png + tile-entry
json per folder).  This script only ever writes its own 52 folders,
itself, and its preview.
"""
from PIL import Image, ImageDraw
import json, math, os, random, zlib

BASE = os.path.dirname(os.path.abspath(__file__))
OUTLINE = (24, 22, 18, 255)

# ---------------------------------------------------------------- colors
def shade(c, f):
    return tuple(max(0, min(255, int(v * f))) for v in c[:3]) + (255,)

# ---------------------------------------------------------------- geometry
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
}
TAIL_ATTACH = {"std": (0.77, 0.455, 0.565), "deep": (0.78, 0.455, 0.565)}
EYE = {"std": (0.145, 0.455), "deep": (0.225, 0.42)}
GILL = {"std": 0.225, "deep": 0.315}

def tail_points(kind, plan):
    ax, tt, tb = TAIL_ATTACH[plan]
    cy = (tt + tb) / 2.0
    if kind == "forked":
        return [(ax, tt), (0.985, tt - 0.135), (0.90, cy), (0.985, tb + 0.135), (ax, tb)]
    if kind == "crescent":
        return [(ax, tt), (0.99, tt - 0.20), (0.895, cy), (0.99, tb + 0.20), (ax, tb)]
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
    rng = random.Random(zlib.crc32(cfg["id"].encode()))

    # --- tail + rear fins (behind body)
    fin_poly(d, tail_points(cfg.get("tail", "forked"), plan), fin, size)
    dp = dorsal_pts(cfg.get("dorsal", "single"), plan, top)
    if dp:
        fin_poly(d, dp, fin, size)
    if cfg.get("adipose"):
        ty = lambda x: interp(top, x)
        fin_poly(d, [(0.665, ty(0.665)), (0.695, ty(0.695) - 0.038), (0.735, ty(0.735))],
                 shade(fin, 0.9), size)
    # anal + pelvic fins (behind body)
    by = lambda x: interp(bot, x)
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
            else:
                c = belly
            px[xi, yi] = c

    pat = cfg.get("pattern")
    if pat == "spots":
        col = cfg.get("pat_color", shade(back, 0.55))
        for _ in range(cfg.get("pat_n", 46)):
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

    # --- head details
    gx = GILL[plan]
    gy0, gy1 = interp(top, gx), interp(bot, gx)
    d.line([(gx + 0.012) * size, (gy0 + 0.02) * size, (gx - 0.008) * size, (gy1 - 0.02) * size],
           fill=shade(flank, 0.62), width=max(1, size // 32))
    ex, ey = EYE[plan]
    er = size * 0.042
    exc, eyc = ex * size, ey * size
    d.ellipse([exc - er, eyc - er, exc + er, eyc + er], fill=(235, 232, 210, 255))
    pr = er * 0.62
    d.ellipse([exc - pr, eyc - pr, exc + pr, eyc + pr], fill=(18, 14, 10, 255))
    mx, my = top[0][0], (top[0][1] + bot[-1][1]) / 2.0
    d.line([mx * size, my * size, (mx + 0.062) * size, (my - 0.016) * size],
           fill=OUTLINE, width=max(1, size // 32))
    # pectoral fin over the body, behind the gill
    pect = [(gx + 0.005, gy0 + 0.16 * (gy1 - gy0)), (gx + 0.095, gy0 + 0.30 * (gy1 - gy0)),
            (gx + 0.075, gy0 + 0.52 * (gy1 - gy0)), (gx + 0.005, gy0 + 0.42 * (gy1 - gy0))]
    fin_poly(d, pect, shade(fin, 0.92), size)
    if cfg.get("jet_tubes"):
        # leaper salmon: two flesh-colored water-jet tubes beneath the fins,
        # angled back along the belly, dark nozzles at the rear ends.
        flesh = (219, 160, 138, 255); flesh_d = shade(flesh, 0.68)
        for tx, ty in ((0.315, 0.685), (0.545, 0.70)):
            x0, y0 = tx * size, ty * size
            x1, y1 = (tx + 0.125) * size, (ty + 0.018) * size
            d.line([x0, y0, x1, y1], fill=flesh_d, width=max(2, size // 9))
            d.line([x0, y0, x1, y1], fill=flesh, width=max(1, size // 15))
            r = max(1.2, size * 0.016)
            d.ellipse([x1 - r, y1 - r, x1 + r, y1 + r], fill=(30, 22, 20, 255))
    return img

# ---------------------------------------------------------------- fry
def draw_fry(size, cfg):
    """Minuscule half-translucent larva (fish-A design) with seeded jitter.

    cfg: id (seed), big (mutant fry: much larger in the tile, pink yolk).
    Translucency is faked with pale opaque colors."""
    rng = random.Random(zlib.crc32(cfg["id"].encode()))
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    big = cfg.get("big", False)
    s = 1.5 if big else rng.uniform(0.92, 1.08)
    cx0, cy0 = 0.465, 0.5
    T = lambda ps: [(cx0 + (x - cx0) * s, cy0 + (y - cy0) * s) for x, y in ps]
    S = lambda ps: [(x * size, y * size) for x, y in ps]
    jt = lambda c, amt: tuple(max(0, min(255, v + rng.randint(-amt, amt))) for v in c[:3]) + (255,)
    pale = jt((198, 214, 218, 255), 6 if big else 10)
    paler = jt((226, 236, 238, 255), 5 if big else 8)
    yolk = (240, 178, 160, 255) if big else jt((238, 204, 152, 255), 12)
    fin_poly(d, T([(0.60, 0.475), (0.71, 0.435), (0.665, 0.50), (0.71, 0.565), (0.60, 0.525)]),
             paler, size)
    body = [(0.315, 0.50), (0.36, 0.455), (0.48, 0.445), (0.575, 0.465), (0.615, 0.50),
            (0.575, 0.535), (0.48, 0.555), (0.36, 0.545)]
    grown = [(cx0 + (x - cx0) * 1.10, cy0 + (y - cy0) * 1.22) for x, y in T(body)]
    d.polygon(S(grown), fill=shade(pale, 0.8))
    d.polygon(S(T(body)), fill=pale)
    yw = 0.06 * s * rng.uniform(0.9, 1.1)
    d.ellipse([(0.415 - yw) * size, (0.557 - yw * 0.72) * size,
               (0.415 + yw) * size, (0.557 + yw * 0.72) * size], fill=yolk)  # yolk sac
    er = size * 0.052 * s
    exc, eyc = (cx0 + (0.375 - cx0) * s) * size, (cy0 + (0.485 - cy0) * s) * size
    d.ellipse([exc - er, eyc - er, exc + er, eyc + er], fill=(240, 244, 246, 255))
    pr = er * 0.66
    d.ellipse([exc - pr, eyc - pr, exc + pr, eyc + pr], fill=(20, 24, 26, 255))
    return img

# ---------------------------------------------------------------- crayfish larva
def draw_crayfish_larva(size):
    """Minuscule half-translucent crustacean larva (zoea-like), centered
    small in the tile like the fry: stalked eyes, a dorsal spine, a
    segmented curl ending in a tail fan, two minute claw buds and
    hair-thin antennae.  Pale opaque colors fake the translucency."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    S = lambda ps: [(x * size, y * size) for x, y in ps]
    pale = (206, 220, 210, 255); dark = shade(pale, 0.62); light = (232, 240, 232, 255)
    w = max(1, size // 32)
    # hair-thin antennae sweeping forward
    for sgn, reach in ((1, 0.16), (-1, 0.12)):
        d.line(S([(0.40, 0.47), (0.33, 0.47 - sgn * 0.05), (0.27, 0.47 - sgn * reach)]),
               fill=dark, width=w)
    # segmented abdomen curling from the thorax back/down to the tail fan
    segs = [(0.545, 0.475, 0.052), (0.615, 0.50, 0.046), (0.67, 0.535, 0.039),
            (0.705, 0.575, 0.032)]
    for cx, cy, r in reversed(segs):
        d.ellipse([(cx - r) * size, (cy - r * 0.8) * size, (cx + r) * size, (cy + r * 0.8) * size],
                  fill=dark)
        ri = r * 0.8
        d.ellipse([(cx - ri) * size, (cy - ri * 0.8) * size, (cx + ri) * size, (cy + ri * 0.8) * size],
                  fill=pale)
    fin_poly(d, [(0.715, 0.565), (0.775, 0.53), (0.785, 0.60), (0.765, 0.665), (0.71, 0.615)],
             light, size)  # tail fan
    # cephalothorax: rounded shield with a forward dorsal spine (zoea)
    shield = [(0.345, 0.475), (0.385, 0.40), (0.47, 0.375), (0.545, 0.415), (0.565, 0.475),
              (0.545, 0.535), (0.47, 0.565), (0.385, 0.545)]
    d.polygon(S([(0.455 + (x - 0.455) * 1.08, 0.475 + (y - 0.475) * 1.10) for x, y in shield]),
              fill=dark)
    d.polygon(S(shield), fill=pale)
    d.polygon(S([(0.455, 0.385), (0.435, 0.315), (0.475, 0.375)]), fill=dark)  # dorsal spine
    d.ellipse([0.40 * size, 0.41 * size, 0.50 * size, 0.485 * size], fill=light)  # highlight
    # stalked eyes
    for ex in (0.385, 0.415):
        d.line(S([(ex, 0.435), (ex - 0.008, 0.40)]), fill=dark, width=w)
        r = size * 0.018
        d.ellipse([(ex - 0.008) * size - r, 0.40 * size - r, (ex - 0.008) * size + r, 0.40 * size + r],
                  fill=(16, 14, 12, 255))
    # two minute claw buds under the head + thread legs
    for cx, cy in ((0.375, 0.565), (0.43, 0.585)):
        d.ellipse([(cx - 0.02) * size, (cy - 0.016) * size, (cx + 0.02) * size, (cy + 0.016) * size],
                  fill=dark)
        d.ellipse([(cx - 0.014) * size, (cy - 0.011) * size, (cx + 0.014) * size, (cy + 0.011) * size],
                  fill=pale)
    for lx in (0.47, 0.52):
        d.line(S([(lx, 0.56), (lx - 0.012, 0.63)]), fill=dark, width=w)
    return img

# ---------------------------------------------------------------- species
FRY = [
    "mon_fry_bluefin_tuna", "mon_fry_bluefish", "mon_fry_bluegill",
    "mon_fry_bowfin", "mon_fry_brook_trout", "mon_fry_brown_trout",
    "mon_fry_bullhead", "mon_fry_bullhead_yellow", "mon_fry_burbot",
    "mon_fry_calico_bass", "mon_fry_carp", "mon_fry_channel_catfish",
    "mon_fry_cod", "mon_fry_fallfish", "mon_fry_flounder",
    "mon_fry_gar_longnose", "mon_fry_goldfish", "mon_fry_grass_carp",
    "mon_fry_green_sunfish", "mon_fry_haddock", "mon_fry_halibut",
    "mon_fry_kokanee_salmon", "mon_fry_lake_trout", "mon_fry_lbass",
    "mon_fry_mackerel", "mon_fry_muskellunge", "mon_fry_pbass",
    "mon_fry_perch", "mon_fry_pickerel", "mon_fry_pickerel_american",
    "mon_fry_pike", "mon_fry_pollock", "mon_fry_porgy",
    "mon_fry_pumpkinseed", "mon_fry_rainbow_trout", "mon_fry_redbreast",
    "mon_fry_rock_bass", "mon_fry_salmon", "mon_fry_sbass",
    "mon_fry_sturgeon_lake", "mon_fry_sturgeon_shortnose",
    "mon_fry_sunfish", "mon_fry_walleye", "mon_fry_warmouth",
    "mon_fry_white_catfish", "mon_fry_white_sucker", "mon_fry_whitefish",
    "mon_fry_whitefish_round",
]
# Adult mutant salmon: mon_fish_salmon's body plan washed to white_magenta.
SALMON_MAGENTA = dict(back=(96, 78, 110, 255), flank=(196, 168, 188, 255),
                      belly=(246, 234, 238, 255), fin=(156, 96, 134, 255),
                      tail="forked", dorsal="single", adipose=True,
                      pattern="spots", pat_color=(58, 42, 66, 255), pat_all=True)

CREDITS = ("Drawn from the monster's in-game description for the CDDA Voxel\n"
           "coverage audit (2026-11-15). License: CC BY-SA 3.0, as the tileset.\n")

def jobs():
    """(id, sheet dir, canvas size, render callable) for the whole batch."""
    out = []
    for mid in FRY:
        out.append((mid, "pngs_normal_32x32", 32,
                    lambda size, mid=mid: draw_fry(size, {"id": mid})))
    out.append(("mon_fry_crayfish", "pngs_normal_32x32", 32,
                lambda size: draw_crayfish_larva(size)))
    out.append(("mon_fry_mutant_salmon", "pngs_normal_32x32", 32,
                lambda size: draw_fry(size, {"id": "mon_fry_mutant_salmon", "big": True})))
    huge = dict(SALMON_MAGENTA, id="mon_mutant_salmon_huge", hscale=1.18, pat_n=60)
    out.append(("mon_mutant_salmon_huge", "pngs_large_64x64", 64,
                lambda size: draw_fish(size, huge)))
    mega = dict(SALMON_MAGENTA, id="mon_mutant_salmon_mega", hscale=1.30, pat_n=72,
                jet_tubes=True)
    out.append(("mon_mutant_salmon_mega", "pngs_large_64x64", 64,
                lambda size: draw_fish(size, mega)))
    return out

def main():
    done = []
    for mid, sheet, size, render in jobs():
        rel = f"{sheet}/monsters/{mid}/{mid}"
        folder = os.path.join(BASE, os.path.dirname(rel))
        os.makedirs(folder, exist_ok=True)
        img = render(size)
        img.save(os.path.join(BASE, rel + ".png"))
        with open(os.path.join(BASE, rel + ".json"), "w") as f:
            json.dump([{"id": mid, "fg": mid, "bg": ""}], f, indent=2)
            f.write("\n")
        with open(os.path.join(folder, "credits.txt"), "w") as f:
            f.write(CREDITS)
        done.append((mid, img))
        print("wrote", rel + ".png", img.size)
    # contact-sheet preview: 8 columns, every sprite shown at 64px
    cols, cell = 8, 68
    rows = (len(done) + cols - 1) // cols
    sheet_img = Image.new("RGBA", (cols * cell, rows * cell), (44, 52, 60, 255))
    for i, (mid, img) in enumerate(done):
        big = img.resize((64, 64), Image.NEAREST) if img.size[0] != 64 else img
        sheet_img.alpha_composite(big, ((i % cols) * cell + 2, (i // cols) * cell + 2))
    sheet_img.save(os.path.join(BASE, "preview_fish_b.png"))
    print("wrote preview_fish_b.png", sheet_img.size, f"({len(done)} sprites)")

if __name__ == "__main__":
    main()
