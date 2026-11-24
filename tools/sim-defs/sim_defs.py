#!/usr/bin/env python3
"""sim-defs — Phase 11 slice S1a: full CDDA def importer for the sim-server.

Reads the pinned CDDA JSON data (fork-src/cdda/data/json, CDDA
6a3e59e0) and produces the full TileDefs table in wire terms
(schema/cdda_ipc.fbs `TileDef`), mirroring the semantics of the
fork's real adapter (fork/src/voxel_ipc/voxel_real_adapter.cpp
`tile_defs()` / `hot_flags_of()` / `item_wire_id()`).

Subcommands:
  census    Walk all JSON, count def-relevant entries -> generated/census.json
  build     Resolve inheritance and emit the table    -> generated/defs.json
            (+ generated/build_report.json)
  validate  Recompute everything and check defs.json  -> generated/validation.json
            (exit non-zero on any un-allowlisted violation;
             --update-known-dangling rewrites generated/known_dangling.json
             from the current findings, after human inspection)

Python 3 stdlib only. See README.md for the documented deviations
(per-kind sorted ids, fg/bg = -1 in this slice) and rules.
"""

import argparse
import copy
import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
DEFAULT_DATA = REPO / "fork-src" / "cdda" / "data" / "json"
DEFAULT_MAPDATA = REPO / "fork-src" / "cdda" / "src" / "mapdata.h"
DEFAULT_CONNECT = DEFAULT_DATA / "connect_groups.json"
DEFAULT_OUT = HERE / "generated"

# JSON "type" -> (wire TileKind, kind name). TileKind per schema:
# Terrain=0, Furniture=1, Trap=2, Field=3, Monster=4, Item=5, VehiclePart=6.
TYPE_TO_KIND = {
    "terrain": (0, "Terrain"),
    "furniture": (1, "Furniture"),
    "trap": (2, "Trap"),
    "field_type": (3, "Field"),
    "MONSTER": (4, "Monster"),
    "ITEM": (5, "Item"),
    "vehicle_part": (6, "VehiclePart"),
}
KIND_NAME = {v[0]: v[1] for v in TYPE_TO_KIND.values()}
TER_FURN_KINDS = {0, 1}  # hot_flags / connect groups / bash exist only here

LOAD_DIRECTIVES = ("copy-from", "abstract", "extend", "delete",
                   "relative", "proportional")


# --------------------------------------------------------------------------
# Source loading
# --------------------------------------------------------------------------

def walk_json(data_dir):
    """Yield (relpath, parsed) for every *.json under data_dir.

    Returns (entries_by_file, parse_failures, non_list_files) where
    entries_by_file is a list of (relpath, list). Files that fail to
    parse or whose top level is not a list are reported, never
    silently skipped.
    """
    files, failures, non_list = [], [], []
    for path in sorted(Path(data_dir).rglob("*.json")):
        rel = str(path.relative_to(data_dir))
        try:
            with open(path, encoding="utf-8") as fh:
                parsed = json.load(fh)
        except Exception as exc:  # noqa: BLE001 - report, don't crash
            failures.append({"file": rel, "error": str(exc)[:200]})
            continue
        if not isinstance(parsed, list):
            non_list.append(rel)
            continue
        files.append((rel, parsed))
    return files, failures, non_list


def collect_entries(data_dir):
    """All dict entries grouped by JSON type, with source file kept."""
    files, failures, non_list = walk_json(data_dir)
    by_type = defaultdict(list)  # type -> [(relpath, entry)]
    total_entries = 0
    for rel, parsed in files:
        for entry in parsed:
            total_entries += 1
            if isinstance(entry, dict):
                by_type[entry.get("type", "<none>")].append((rel, entry))
    return by_type, {
        "files_read": len(files),
        "total_entries": total_entries,
        "parse_failures": failures,
        "non_list_files": non_list,
    }


def compute_census(data_dir):
    by_type, meta = collect_entries(data_dir)
    types = {}
    for jtype in sorted(TYPE_TO_KIND):
        rows = by_type.get(jtype, [])
        types[jtype] = {
            "kind": TYPE_TO_KIND[jtype][0],
            "kind_name": TYPE_TO_KIND[jtype][1],
            "total": len(rows),
            "copy_from": sum(1 for _, e in rows if "copy-from" in e),
            "abstract": sum(1 for _, e in rows if "abstract" in e),
            "id_less": sum(1 for _, e in rows if "id" not in e),
            # In this dataset every abstract entry is also id-less
            # (abstract entries carry "abstract": "<id>" instead of
            # "id"); the excludable set is the UNION, computed here so
            # validation never double-subtracts.
            "abstract_or_id_less": sum(
                1 for _, e in rows if "abstract" in e or "id" not in e),
            "extend": sum(1 for _, e in rows if "extend" in e),
            "relative": sum(1 for _, e in rows if "relative" in e),
            "proportional": sum(1 for _, e in rows if "proportional" in e),
        }
    return {"source": str(data_dir), **meta, "types": types}


# --------------------------------------------------------------------------
# Reference tables parsed from the pinned game source (never hardcoded)
# --------------------------------------------------------------------------

def parse_ter_furn_flags(mapdata_h):
    """Enum order of `enum class ter_furn_flag` from mapdata.h."""
    text = Path(mapdata_h).read_text(encoding="utf-8")
    m = re.search(r"enum class ter_furn_flag[^{]*\{(.*?)\};", text, re.S)
    if not m:
        raise SystemExit(f"ter_furn_flag enum not found in {mapdata_h}")
    names = []
    for line in m.group(1).splitlines():
        line = line.split("//")[0].strip().rstrip(",")
        if re.fullmatch(r"TFLAG_[A-Z0-9_]+", line):
            names.append(line)
    if "NUM_TFLAG_FLAGS" in names:
        names.remove("NUM_TFLAG_FLAGS")
    if not names:
        raise SystemExit("ter_furn_flag enum parsed empty")
    return names  # index in this list == enum value == hot_flags bit


def parse_connect_groups(connect_json):
    """Connect-group registry in load order (order of entries in the file)."""
    data = json.loads(Path(connect_json).read_text(encoding="utf-8"))
    return [e["id"] for e in data
            if isinstance(e, dict) and e.get("type") == "connect_group"
            and "id" in e]


# --------------------------------------------------------------------------
# Inheritance resolution
# --------------------------------------------------------------------------

def resolve_entries(rows):
    """Resolve copy-from chains for one JSON type.

    rows: [(relpath, entry)]. Returns (resolved_by_id, report) where
    resolved_by_id maps string_id -> fully merged entry dict for
    every non-abstract entry that carries an "id".

    Merge rule (documented in README.md): the parent's resolved
    dict is the base; the child overrides field-by-field, except
    that when both parent and child values for a key are dicts they
    merge one level deep (child subkeys win — this is how a child
    can override just `bash.ter_set`). Lists and scalars replace
    wholesale. An "extend" dict is then applied the CDDA way:
    list targets are appended to, dict targets updated, anything
    else set. "relative"/"proportional" are numeric load-time
    adjustments to fields this importer does not emit; they are
    counted in the census and otherwise ignored. Load directives
    (copy-from/abstract/extend/delete/relative/proportional) never
    survive into a resolved entry. Abstract entries are registered
    as copy-from parents under their "abstract" value but are never
    emitted themselves.
    """
    registry = {}   # key (id or abstract value) -> raw entry
    dup_keys = []
    for _, entry in rows:
        key = entry.get("id") or entry.get("abstract")
        if key is None:
            continue
        if key in registry:
            dup_keys.append(key)
        registry[key] = entry

    resolved_cache = {}
    unresolved = []   # (string_id, missing copy-from target)
    cycles = []

    def resolve(key, stack):
        if key in resolved_cache:
            return resolved_cache[key]
        if key in stack:
            cycles.append(list(stack) + [key])
            return {}
        raw = registry.get(key)
        if raw is None:
            return None
        merged = {}
        parent_key = raw.get("copy-from")
        if parent_key is not None:
            parent = resolve(parent_key, stack + (key,))
            if parent is None:
                unresolved.append((raw.get("id") or raw.get("abstract"),
                                   parent_key))
                parent = {}
            merged = copy.deepcopy(parent)
        for field, value in raw.items():
            if field in LOAD_DIRECTIVES:
                continue
            if isinstance(value, dict) and isinstance(merged.get(field), dict):
                combined = dict(merged[field])
                combined.update(copy.deepcopy(value))
                merged[field] = combined
            else:
                merged[field] = copy.deepcopy(value)
        ext = raw.get("extend")
        if isinstance(ext, dict):
            for field, value in ext.items():
                cur = merged.get(field)
                if isinstance(cur, list) and isinstance(value, list):
                    merged[field] = cur + copy.deepcopy(value)
                elif isinstance(cur, dict) and isinstance(value, dict):
                    combined = dict(cur)
                    combined.update(copy.deepcopy(value))
                    merged[field] = combined
                else:
                    merged[field] = copy.deepcopy(value)
        resolved_cache[key] = merged
        return merged

    out = {}
    for _, entry in rows:
        if "abstract" in entry or "id" not in entry:
            continue
        merged = resolve(entry["id"], ())
        if merged is not None:
            merged["id"] = entry["id"]
            merged["type"] = entry["type"]
            out[entry["id"]] = merged
    report = {
        "duplicate_registry_keys": sorted(set(dup_keys)),
        "unresolved_copy_from": [
            {"entry": a, "missing_parent": b} for a, b in unresolved],
        "copy_from_cycles": cycles,
    }
    return out, report


# --------------------------------------------------------------------------
# Def extraction (mirrors voxel_real_adapter.cpp)
# --------------------------------------------------------------------------

def extract_name(entry, string_id):
    """Name in wire form. Handles str and translation dicts
    ({"str": ...} / {"str_sp": ...}); field types take their first
    intensity level's name (the adapter's field_type::name(0)).
    Falls back to the string_id when no name exists at all."""
    name = entry.get("name")
    if isinstance(name, str) and name:
        return name, False
    if isinstance(name, dict):
        for key in ("str", "str_sp", "str_pl"):
            if isinstance(name.get(key), str) and name[key]:
                return name[key], False
    levels = entry.get("intensity_levels")
    if isinstance(levels, list) and levels and isinstance(levels[0], dict):
        lvl_name = levels[0].get("name")
        if isinstance(lvl_name, str) and lvl_name:
            return lvl_name, False
        if isinstance(lvl_name, dict) and lvl_name.get("str"):
            return lvl_name["str"], False
    return string_id, True


def as_str_list(value):
    if isinstance(value, str):
        return [value]
    if isinstance(value, list):
        return [v for v in value if isinstance(v, str)]
    return []


def build_defs(data_dir, mapdata_h, connect_json):
    by_type, meta = collect_entries(data_dir)
    flag_order = parse_ter_furn_flags(mapdata_h)
    flag_index = {name: i for i, name in enumerate(flag_order)}
    connect_names = parse_connect_groups(connect_json)
    connect_index = {name: i for i, name in enumerate(connect_names)}

    unknown_flags = Counter()
    unknown_groups = Counter()
    name_fallbacks = Counter()
    per_kind_reports = {}
    all_defs = []

    for jtype, (kind, kind_name) in sorted(TYPE_TO_KIND.items(),
                                           key=lambda kv: kv[1][0]):
        rows = by_type.get(jtype, [])
        resolved, report = resolve_entries(rows)
        per_kind_reports[jtype] = report
        defs = []
        for string_id in sorted(resolved):
            entry = resolved[string_id]
            name, fell_back = extract_name(entry, string_id)
            if fell_back:
                name_fallbacks[jtype] += 1
            flags = as_str_list(entry.get("flags")) if kind in TER_FURN_KINDS else []
            hot = 0
            if kind in TER_FURN_KINDS:
                for flag in flags:
                    idx = flag_index.get("TFLAG_" + flag)
                    if idx is None:
                        unknown_flags[flag] += 1
                    elif idx < 64:  # adapter truncates at bit 63
                        hot |= 1 << idx

            def group_indices(field):
                out = []
                for gname in as_str_list(entry.get(field)):
                    if gname in connect_index:
                        out.append(connect_index[gname])
                    else:
                        unknown_groups[gname] += 1
                return out

            bash = entry.get("bash")
            bash_result = None
            if kind in TER_FURN_KINDS and isinstance(bash, dict):
                key = "ter_set" if kind == 0 else "furn_set"
                val = bash.get(key)
                if isinstance(val, str) and val:
                    bash_result = val

            def opt_str(field):
                val = entry.get(field)
                return val if isinstance(val, str) and val else None

            defs.append({
                "id": 0,  # assigned below
                "string_id": string_id,
                "name": name,
                "kind": kind,
                "flags": flags,
                "hot_flags": hot,
                "connect_groups": group_indices("connect_groups") if kind in TER_FURN_KINDS else [],
                "connects_to_groups": group_indices("connects_to") if kind in TER_FURN_KINDS else [],
                "rotate_to_groups": group_indices("rotates_to") if kind in TER_FURN_KINDS else [],
                "open_result": opt_str("open") if kind in TER_FURN_KINDS else None,
                "close_result": opt_str("close") if kind in TER_FURN_KINDS else None,
                "bash_result": bash_result,
                "looks_like": opt_str("looks_like"),
                "roof_result": opt_str("roof") if kind == 0 else None,
                # fg/bg join is a documented follow-up; -1 = none,
                # never invented (the real adapter does the same
                # when its refs table is absent).
                "fg_ref": -1,
                "bg_ref": -1,
            })
        # Per-kind ids: sorted string_ids, 1..N (0 = none). The real
        # adapter emits the game's own load-order int ids (and items
        # already use sorted order there — item_wire_id()); game int
        # ids are not reproducible from JSON alone. See README.
        for i, d in enumerate(defs, start=1):
            d["id"] = i
        all_defs.extend(defs)

    build_report = {
        "source_meta": meta,
        "ter_furn_flag_count": len(flag_order),
        "connect_group_count": len(connect_names),
        "unknown_flags": dict(sorted(unknown_flags.items())),
        "unknown_connect_groups": dict(sorted(unknown_groups.items())),
        "name_fallback_to_string_id": dict(name_fallbacks),
        "per_kind": per_kind_reports,
        "counts_per_kind": {
            KIND_NAME[k]: sum(1 for d in all_defs if d["kind"] == k)
            for k in sorted(KIND_NAME)},
    }
    table = {
        "protocol_version": 1,
        "source": "CDDA master @ 6a3e59e0 (fork-src/cdda/data/json)",
        "connect_groups": connect_names,
        "defs": all_defs,
    }
    return table, build_report


# --------------------------------------------------------------------------
# Validation
# --------------------------------------------------------------------------

TRANSFORM_FIELDS = ("open_result", "close_result", "bash_result",
                    "roof_result", "looks_like")


def find_dangling(defs):
    """Every transform string that does not resolve within its
    expected kind. looks_like is checked within the same kind, per
    the S1a spec; open/close/bash/roof within terrain for terrain
    defs and within furniture for furniture defs (they are only
    emitted for those kinds). Each finding is classified by where
    the target *does* exist, to make human inspection fast."""
    ids_by_kind = defaultdict(set)
    all_ids = {}
    for d in defs:
        ids_by_kind[d["kind"]].add(d["string_id"])
        all_ids.setdefault(d["string_id"], []).append(d["kind"])
    findings = []
    for d in defs:
        for field in TRANSFORM_FIELDS:
            target = d.get(field)
            if not target:
                continue
            if target in ids_by_kind[d["kind"]]:
                continue
            if target in all_ids:
                reason = ("target exists in another kind "
                          f"({[KIND_NAME[k] for k in all_ids[target]]}); "
                          "string ids are global in the game, per-kind "
                          "resolution is the sim's stricter rule")
            elif target in ("t_null", "f_null"):
                reason = ("target is the game's built-in null id "
                          "(mapdata.cpp set_ter_ids: ter_id \"t_null\" "
                          "is the null terrain sentinel, id 0 = none); "
                          "it has no JSON def by design")
            else:
                reason = ("target not present in any imported kind. "
                          "Inspected 2026-11-23: genuine dangling "
                          "references in the pinned upstream data — "
                          "legacy/removed ids (t_machinery_electronic, "
                          "t_plut_generator, t_foam_roof), tileset-only "
                          "sprite names used as monster looks_like "
                          "(chud, mon_frog), ids that now exist only as "
                          "MIGRATION entries (fridge, armor_lightplate), "
                          "and one whitespace looks_like (\" \") on the "
                          "t_deck_coating_no_roof_* terrains")
            findings.append({
                "kind": d["kind"], "kind_name": KIND_NAME[d["kind"]],
                "string_id": d["string_id"], "field": field,
                "target": target, "classification": reason,
            })
    return findings


def spot_checks(defs):
    by_id = {(d["kind"], d["string_id"]): d for d in defs}
    checks = []

    door = by_id.get((0, "t_door_c"), {})
    checks.append({
        "check": "t_door_c.open_result == t_door_o",
        "actual_open_result": door.get("open_result"),
        "pass": door.get("open_result") == "t_door_o"})
    checks.append({
        "check": "t_door_c.roof_result non-empty",
        "actual_roof_result": door.get("roof_result"),
        "pass": bool(door.get("roof_result"))})
    glass = {sid: d for (k, sid), d in by_id.items()
             if k == 0 and "door_glass_c" in sid}
    glass_hits = {sid: d["bash_result"] for sid, d in glass.items()
                  if d.get("bash_result") == "t_mdoor_frame"}
    checks.append({
        "check": "a laminated/ballistic glass door bashes to t_mdoor_frame",
        "glass_doors_found": sorted(glass),
        "bash_to_t_mdoor_frame": glass_hits,
        "pass": len(glass_hits) > 0})
    floor = by_id.get((0, "t_floor"), {})
    checks.append({
        "check": "t_floor.roof_result present",
        "actual_roof_result": floor.get("roof_result"),
        "pass": bool(floor.get("roof_result"))})
    return checks


def run_validate(data_dir, defs_path, known_dangling_path,
                 update_known_dangling=False):
    census = compute_census(data_dir)
    table = json.loads(Path(defs_path).read_text(encoding="utf-8"))
    defs = table["defs"]
    failures = []

    # -- counts: recomputed from source, never from build output.
    # Expected = census total - |abstract UNION id-less| (in this
    # dataset the two sets are identical: abstracts carry no "id").
    count_checks = {}
    for jtype, (kind, kind_name) in TYPE_TO_KIND.items():
        c = census["types"][jtype]
        expected = c["total"] - c["abstract_or_id_less"]
        actual = sum(1 for d in defs if d["kind"] == kind)
        ok = expected == actual
        count_checks[jtype] = {
            "census_total": c["total"], "abstract": c["abstract"],
            "id_less": c["id_less"],
            "abstract_or_id_less": c["abstract_or_id_less"],
            "expected": expected, "actual": actual, "pass": ok}
        if not ok:
            failures.append(
                f"count mismatch {jtype}: expected {expected}, got {actual}")

    # -- abstracts / ids / string_ids
    if any("abstract" in d for d in defs):
        failures.append("abstract marker leaked into output defs")
    seen = set()
    for d in defs:
        if not d.get("string_id"):
            failures.append(f"def without string_id: {d}")
        key = (d["kind"], d["id"])
        if key in seen:
            failures.append(f"duplicate id within kind: {key}")
        seen.add(key)
        skey = (d["kind"], d["string_id"])
        # string_id uniqueness within kind follows from build, check anyway
    sids = Counter((d["kind"], d["string_id"]) for d in defs)
    for skey, n in sids.items():
        if n > 1:
            failures.append(f"duplicate string_id within kind: {skey} x{n}")

    # -- connect registry parity
    registry_ok = table.get("connect_groups") == parse_connect_groups(
        DEFAULT_CONNECT if data_dir == DEFAULT_DATA else
        Path(data_dir) / "connect_groups.json")
    if not registry_ok:
        failures.append("defs.json connect_groups != connect_groups.json load order")

    # -- dangling transforms
    findings = find_dangling(defs)
    known = []
    if Path(known_dangling_path).exists():
        known_doc = json.loads(Path(known_dangling_path).read_text(encoding="utf-8"))
        known = known_doc.get("known_dangling", [])
    known_keys = {(k["kind"], k["string_id"], k["field"], k["target"])
                  for k in known}
    finding_keys = {(f["kind"], f["string_id"], f["field"], f["target"])
                    for f in findings}
    unlisted = [f for f in findings
                if (f["kind"], f["string_id"], f["field"], f["target"])
                not in known_keys]
    stale = sorted(known_keys - finding_keys)
    if update_known_dangling:
        doc = {
            "comment": ("Upstream dangling transform references, "
                        "inspected 2026-11-23. These exist in the "
                        "pinned CDDA data itself (see each entry's "
                        "classification); the sim importer reports "
                        "them rather than inventing targets. Any NEW "
                        "dangling reference not in this list fails "
                        "validation."),
            "known_dangling": findings,
        }
        Path(known_dangling_path).write_text(
            json.dumps(doc, indent=1) + "\n", encoding="utf-8")
        known = findings
        unlisted = []
    if unlisted:
        failures.append(f"{len(unlisted)} dangling transform(s) not in "
                        "known-dangling allowlist")
    if stale:
        failures.append(f"{len(stale)} known-dangling entr(ies) no longer "
                        "dangle (allowlist is stale)")

    # -- spot checks
    spots = spot_checks(defs)
    for s in spots:
        if not s["pass"]:
            failures.append(f"spot check failed: {s['check']}")

    validation = {
        "verdict": "PASS" if not failures else "FAIL",
        "failures": failures,
        "count_checks": count_checks,
        "total_defs": len(defs),
        "dangling_total": len(findings),
        "dangling_allowlisted": len(findings) - len(unlisted),
        "dangling_unlisted": unlisted,
        "dangling_stale_allowlist_entries": [list(k) for k in stale],
        "dangling_by_field": dict(Counter(f["field"] for f in findings)),
        "dangling_by_target": dict(Counter(f["target"] for f in findings)
                                   .most_common()),
        "spot_checks": spots,
    }
    return validation


# --------------------------------------------------------------------------
# CLI
# --------------------------------------------------------------------------

def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--data", default=str(DEFAULT_DATA))
    ap.add_argument("--mapdata", default=str(DEFAULT_MAPDATA))
    ap.add_argument("--out-dir", default=str(DEFAULT_OUT))
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("census")
    sub.add_parser("build")
    vp = sub.add_parser("validate")
    vp.add_argument("--update-known-dangling", action="store_true",
                    help="rewrite known_dangling.json from current "
                         "findings (only after inspecting them)")
    args = ap.parse_args(argv)

    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    data_dir = Path(args.data)
    connect_json = data_dir / "connect_groups.json"

    if args.cmd == "census":
        census = compute_census(data_dir)
        (out_dir / "census.json").write_text(
            json.dumps(census, indent=1) + "\n", encoding="utf-8")
        for jtype, c in census["types"].items():
            print(f"{jtype:14s} total={c['total']:6d} "
                  f"copy-from={c['copy_from']:5d} abstract={c['abstract']:4d} "
                  f"id-less={c['id_less']:4d}")
        print(f"files={census['files_read']} "
              f"parse_failures={len(census['parse_failures'])} "
              f"non_list={len(census['non_list_files'])}")
        return 0

    if args.cmd == "build":
        table, report = build_defs(data_dir, args.mapdata, connect_json)
        (out_dir / "defs.json").write_text(
            json.dumps(table, indent=1) + "\n", encoding="utf-8")
        (out_dir / "build_report.json").write_text(
            json.dumps(report, indent=1) + "\n", encoding="utf-8")
        print("counts per kind:", report["counts_per_kind"])
        print(f"unknown flags: {len(report['unknown_flags'])} distinct, "
              f"unknown connect groups: "
              f"{len(report['unknown_connect_groups'])} distinct")
        return 0

    if args.cmd == "validate":
        validation = run_validate(
            data_dir, out_dir / "defs.json",
            out_dir / "known_dangling.json",
            update_known_dangling=args.update_known_dangling)
        (out_dir / "validation.json").write_text(
            json.dumps(validation, indent=1) + "\n", encoding="utf-8")
        print(f"verdict: {validation['verdict']} "
              f"(defs={validation['total_defs']}, "
              f"dangling={validation['dangling_total']} total / "
              f"{len(validation['dangling_unlisted'])} unlisted)")
        for f in validation["failures"]:
            print("FAIL:", f)
        return 0 if validation["verdict"] == "PASS" else 1

    return 2


if __name__ == "__main__":
    sys.exit(main())
