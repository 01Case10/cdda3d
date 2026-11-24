#!/usr/bin/env python3
"""
Audit base-game CDDA ids against the UltiCa tileset. v2.

Two tileset universes are measured, because they differ:

* **json** -- tile-entry JSON ids only. This is what our voxelizer's
  decomposed loader (crates/tileset) currently sees.
* **shipped** -- json ids UNION the stems of PNGs that no tile entry
  references. CDDA's official tileset build composes UltiCa with
  `compose.py --use-all` (.github/workflows/compose-tilesets.yml), and
  --use-all turns every *unreferenced* PNG into a tile entry
  {id: <png stem>, fg: <that sprite>}. Orphan PNGs such as
  mon_antlion_larva.png (a PNG with no .json at all) therefore ARE
  represented in the shipped game. (v1 of this audit unioned *every*
  PNG stem; that overcounted ids whose stem-named PNG is referenced by
  some other entry -- compose creates no entry for those. Corrected
  2026-11-15, the same day crates/tileset gained matching synthesis.)

Resolution chain (crates/tileset/src/resolve.rs, mirroring cata_tiles):
  exact -> variant (<id>_var_*) -> season (<id>_season_*) -> looks_like
  (recursive, jump limit 10, looks_like inherited through copy-from)
  -> filler.  Filler counts as NOT represented (Phase 5 decision:
  filler produces the magenta/black checker placeholder).

Game universe: fork-src/cdda/data/json/** (core only, not data/mods),
pinned CDDA 6a3e59e0. Tileset: Tilesets 5ee942f9, gfx/UltimateCataclysm.
[Base note, 2026-11-23: the fork base moved to CDDA 0.I-1 (7b2efa5c)
per plan.md Addendum 12, but this tool reads fork-src/cdda, which
still holds the master checkout — its output remains a master-pin
coverage record until fork-src is replaced and the audit re-run
(Addendum 12 step 5).]

vehicle_part ids have no prefix in game JSON; their tileset entries are
`vp_<id>` plus facing/edge suffixes (see CDDA's own
tools/gfx_tools/tileset_coverage.py strip logic) -- handled specially.

Outputs (coverage-audit/): missing.json, coverage.md
"""
import json, os, glob, collections, re

ROOT = os.path.expanduser("~/workspace/cdda-voxel")
GAME_JSON = os.path.join(ROOT, "fork-src/cdda/data/json")
TILESET = os.path.join(ROOT, "fork-src/tilesets/gfx/UltimateCataclysm")
OUT = os.path.join(ROOT, "coverage-audit")
SEASONS = ["spring", "summer", "autumn", "winter"]

# ---------- tileset universes ----------
json_ids = set()
for path in glob.glob(os.path.join(TILESET, "**/*.json"), recursive=True):
    if os.path.basename(path) in ("tile_info.json", "layering.json"):
        continue
    try:
        data = json.load(open(path, encoding="utf-8", errors="replace"))
    except Exception:
        continue
    for it in (data if isinstance(data, list) else [data]):
        if not isinstance(it, dict) or "id" not in it:
            continue
        ids = it["id"] if isinstance(it["id"], list) else [it["id"]]
        for i in ids:
            if isinstance(i, str):
                json_ids.add(i)

png_stems = set()
for path in glob.glob(os.path.join(TILESET, "pngs_*", "**/*.png"), recursive=True):
    png_stems.add(os.path.splitext(os.path.basename(path))[0])

# Sprite names referenced by any tile entry (fg/bg, incl. additional_tiles
# and weighted/list forms) -- compose.py's "referenced" set for --use-all.
referenced = set()
def collect_refs(node):
    if isinstance(node, str):
        referenced.add(node)
    elif isinstance(node, list):
        for x in node:
            collect_refs(x)
    elif isinstance(node, dict):
        if isinstance(node.get("sprite"), str):
            referenced.add(node["sprite"])
        for k in ("fg", "bg"):
            if k in node:
                collect_refs(node[k])
        for sub in node.get("additional_tiles", []) or []:
            collect_refs(sub)
for path in glob.glob(os.path.join(TILESET, "**/*.json"), recursive=True):
    if os.path.basename(path) in ("tile_info.json", "layering.json"):
        continue
    try:
        data = json.load(open(path, encoding="utf-8", errors="replace"))
    except Exception:
        continue
    for it in (data if isinstance(data, list) else [data]):
        if isinstance(it, dict):
            collect_refs(it)

useall_ids = {s for s in png_stems if s not in referenced}
shipped_ids = json_ids | useall_ids

def make_index(ids):
    variant_bases, season_bases = set(), set()
    for t in ids:
        if "_var_" in t:
            variant_bases.add(t.split("_var_")[0])
        for s in SEASONS:
            suf = f"_season_{s}"
            if t.endswith(suf):
                season_bases.add(t[: -len(suf)])
    return {"ids": ids, "variant_bases": variant_bases, "season_bases": season_bases}

IDX = {"json": make_index(json_ids), "shipped": make_index(shipped_ids)}

def resolve_stage(id_, looks_like_map, idx):
    ids = idx["ids"]
    if id_ in ids:
        return ("exact" if id_ in json_ids or idx is IDX["json"] else "exact/use-all-png", id_)
    if id_ in idx["variant_bases"]:
        return ("variant", id_)
    if id_ in idx["season_bases"]:
        return ("season", id_)
    cur, jumps, seen = id_, 0, set()
    while jumps < 10:
        nxt = looks_like_map.get(cur)
        if not nxt or nxt in seen:
            break
        seen.add(cur); jumps += 1
        if nxt in ids or nxt in idx["variant_bases"] or nxt in idx["season_bases"]:
            return ("looks_like", nxt)
        cur = nxt
    return ("filler/missing", None)

# vehicle-part stripped set (CDDA tileset_coverage.py logic, simplified)
VP_SUFFIX = re.compile(r"(_(cover|cross|horizontal|horizontal_2|vertical|vertical_2|ne|nw|se|sw))?(_(unconnected|left|right|rear|front))?(_edge)?$")
def vp_cover(id_):
    for cand in shipped_ids:
        pass  # replaced by precomputed set below
    return False
vp_stripped = set()
for t in shipped_ids:
    if t.startswith("vp_"):
        vp_stripped.add(VP_SUFFIX.sub("", t[3:]))
    # season variants of vp entries
    for s in SEASONS:
        suf = f"_season_{s}"
        if t.startswith("vp_") and t.endswith(suf):
            vp_stripped.add(VP_SUFFIX.sub("", t[3:-len(suf)]))

# ---------- game objects ----------
CATS = {"terrain": "terrain", "furniture": "furniture", "MONSTER": "monster",
        "vehicle_part": "vehicle_part", "trap": "trap", "field_type": "field_type",
        "ITEM": "item", "overmap_terrain": "overmap_terrain"}
objects, all_by_id = [], {}
def iter_objs(data):
    if isinstance(data, list):
        for x in data:
            yield from iter_objs(x)
    elif isinstance(data, dict):
        yield data

for path in sorted(glob.glob(os.path.join(GAME_JSON, "**/*.json"), recursive=True)):
    try:
        data = json.load(open(path, encoding="utf-8", errors="replace"))
    except Exception:
        continue
    for obj in iter_objs(data):
        if not isinstance(obj.get("type"), str):
            continue
        key = obj.get("id") or obj.get("abstract")
        if isinstance(key, str):
            all_by_id.setdefault(key, obj)
        cat = CATS.get(obj["type"])
        if cat is None or not isinstance(obj.get("id"), str):
            continue
        objects.append({"id": obj["id"], "cat": cat, "type": obj["type"],
                        "raw": obj, "file": os.path.relpath(path, GAME_JSON)})

def inherited(obj, field):
    cur, seen = obj, set()
    while isinstance(cur, dict):
        if field in cur:
            return cur[field]
        cf = cur.get("copy-from")
        if not cf or cf in seen:
            return None
        seen.add(cf)
        cur = all_by_id.get(cf)
    return None

def text_of(v):
    if isinstance(v, str): return v
    if isinstance(v, dict): return v.get("str") or v.get("str_sp") or ""
    return ""

looks_like_map = {}
for o in objects:
    ll = inherited(o["raw"], "looks_like")
    if isinstance(ll, str) and ll:
        looks_like_map[o["id"]] = ll

# ---------- audit ----------
results = collections.defaultdict(list)
for o in objects:
    rec = {"id": o["id"], "category": o["cat"], "type": o["type"],
           "looks_like": looks_like_map.get(o["id"]),
           "name": text_of(inherited(o["raw"], "name")),
           "description": text_of(inherited(o["raw"], "description")),
           "color": inherited(o["raw"], "color"),
           "symbol": inherited(o["raw"], "symbol"),
           "file": o["file"]}
    for uni in ("json", "shipped"):
        stage, resolved = resolve_stage(o["id"], looks_like_map, IDX[uni])
        if o["cat"] == "vehicle_part" and stage == "filler/missing" and o["id"] in vp_stripped:
            stage, resolved = "vp_prefixed", f"vp_{o['id']}"
        rec[f"stage_{uni}"] = stage
        rec[f"resolved_{uni}"] = resolved
    results[o["cat"]].append(rec)

os.makedirs(OUT, exist_ok=True)
missing_shipped, useall_only = [], []
lines = ["# CDDA base-game coverage vs UltiCa tileset (v2)", "",
         "Pinned: CDDA 6a3e59e0 / Tilesets 5ee942f9 (UltiCa).",
         "Base note 2026-11-23: master-pin record — fork base is now CDDA 0.I-1 (7b2efa5c), but this audit reads fork-src/cdda (still the master checkout); re-run after fork-src is replaced (plan.md Addendum 12 step 5).",
         "`json` universe = tile-entry JSON ids (what our voxelizer loader saw before the 2026-11-15 --use-all fix).",
         "`shipped` universe = json ids + stems of PNGs no entry references (compose.py --use-all, the official UltiCa build; our loader now synthesizes the same entries).",
         "Resolution: exact -> variant -> season -> looks_like (copy-from inherited, jump limit 10) -> filler. Filler = NOT represented.", ""]
order = ["terrain", "furniture", "monster", "trap", "field_type", "vehicle_part", "overmap_terrain", "item"]
summary = {}
for cat in order:
    recs = results.get(cat, [])
    if not recs: continue
    mj = [r for r in recs if r["stage_json"] == "filler/missing"]
    ms = [r for r in recs if r["stage_shipped"] == "filler/missing"]
    ua = [r for r in recs if r["stage_json"] == "filler/missing" and r["stage_shipped"] != "filler/missing"]
    missing_shipped.extend(ms); useall_only.extend(ua)
    summary[cat] = {"total": len(recs), "missing_json_loader": len(mj),
                    "missing_shipped_game": len(ms), "useall_png_only": len(ua)}
    lines.append(f"## {cat}: {len(recs)} ids — not represented in shipped game: {len(ms)} ({100.0*len(ms)/len(recs):.1f}%) — not represented to our voxelizer loader: {len(mj)} — of which {len(ua)} are orphan-PNG (--use-all) cases")
    lines.append(f"shipped stages: {dict(collections.Counter(r['stage_shipped'] for r in recs))}")
    lines.append("")
    lines.append("### Truly missing (no JSON entry, no orphan PNG, no looks_like rescue)")
    for r in sorted(ms, key=lambda x: x["id"]):
        lines.append(f"- `{r['id']}` — {r['name'] or '(no name)'} — looks_like: {r['looks_like'] or '-'} — {r['file']}")
    lines.append("")
    lines.append("### Orphan-PNG only (--use-all in shipped game, invisible to our loader)")
    for r in sorted(ua, key=lambda x: x["id"]):
        lines.append(f"- `{r['id']}` — {r['name'] or '(no name)'} — via {r['stage_shipped']} -> {r['resolved_shipped']}")
    lines.append("")

json.dump({"tileset_json_ids": len(json_ids), "tileset_png_stems": len(png_stems),
           "tileset_useall_ids": len(useall_ids),
           "png_stems_referenced_elsewhere": len(png_stems & referenced - json_ids),
           "summary": summary,
           "missing_shipped": sorted(missing_shipped, key=lambda x: (x["category"], x["id"])),
           "useall_png_only": sorted(useall_only, key=lambda x: (x["category"], x["id"]))},
          open(os.path.join(OUT, "missing.json"), "w"), indent=1)
open(os.path.join(OUT, "coverage.md"), "w").write("\n".join(lines))
for cat in order:
    if cat in summary:
        print(cat, summary[cat])
print(f"json_ids={len(json_ids)} png_stems={len(png_stems)} useall={len(useall_ids)} shipped={len(shipped_ids)} referenced_elsewhere={len(png_stems & referenced - json_ids)}")
