#!/usr/bin/env python3
"""Extract the Phase 0 hook-point catalogue from the pinned CDDA source.

For every function the plan depends on, record file, line, and the exact
source line. Output: docs/hook-points.md (human) + tools/phase0/results/
hooks.json (machine). Run after fork-src/cdda is cloned at the pinned
commit (fork base: CDDA 0.I-1, release tag 0.I-1, 7b2efa5c, since
2026-11-23 — plan.md Addendum 12; originally master @ 6a3e59e0).
"""
import json, os, re, sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
CDDA = os.path.join(ROOT, "fork-src", "cdda")

# (area, file, regex, why it matters)
HOOKS = [
    ("Send boundary", "src/do_turn.cpp", r"bool game::do_turn", "primary send boundary candidate: fires once per completed player turn (definition lives in do_turn.cpp, NOT game.cpp — Phase 0 finding)"),
    ("Send boundary", "src/game.cpp", r"void game::draw\( ui_adaptor", "game::draw(ui_adaptor&) — redraw point; panels+terrain drawn here"),
    ("Send boundary", "src/game.h", r"bool do_turn", "do_turn declaration"),
    ("Snapshot read", "src/map.h", r"ter_id ter\( const tripoint_bub_ms", "terrain id per tile"),
    ("Snapshot read", "src/map.h", r"furn_id furn\( tripoint_bub_ms", "furniture id per tile (takes tripoint by value)"),
    ("Snapshot read", "src/map.h", r"const trap &tr_at\( const tripoint_bub_ms", "trap per tile"),
    ("Snapshot read", "src/map.h", r"const field &field_at\( const tripoint_bub_ms", "FULL field map per tile (type->entry)"),
    ("Snapshot read", "src/map.h", r"map_stack i_at\( const tripoint_bub_ms", "item stack per tile"),
    ("Snapshot read", "src/map.h", r"bool has_graffiti_at\( const tripoint_bub_ms", "graffiti flag"),
    ("Snapshot read", "src/submap.h", r"get_uppermost_item", "uppermost item helper — lives on the submap maptile (submap.h), not map"),
    ("Light / FOV", "src/map.h", r"lit_level apparent_light_at", "per-tile lit_level from visibility cache"),
    ("Light / FOV", "src/map.h", r"void update_visibility_cache", "recompute visibility cache per z"),
    ("Light / FOV", "src/lightmap.h", r"enum class lit_level|enum lit_level", "lit_level enum definition"),
    ("Delta dirty set", "src/map.cpp", r"void map::invalidate_map_cache", "coarse invalidation hook (per z)"),
    ("Delta dirty set", "src/map.h", r"set_seen_cache_dirty", "seen-cache dirty hook"),
    ("Delta dirty set", "src/map.h", r"set_transparency_cache_dirty", "transparency-cache dirty hook"),
    ("Delta dirty set", "src/map.cpp", r"void map::shift\( const point_rel_sm", "bubble recentering — snapshot epoch event"),
    ("UI capture", "src/sdltiles.cpp", r"display_buffer", "SDL backend's single composited texture (Route A capture point)"),
    ("UI capture", "src/game.cpp", r"draw_ter\(\)", "terrain draw — suppressed in frontend mode (Route A)"),
    ("UI capture", "src/game.cpp", r"draw_panels", "sidebar/message panels drawn after terrain"),
    ("UI capture", "src/cursesport.cpp", r"catacurses|WINDOW", "catacurses window impl (Route B capture layer)"),
    ("Input inject", "src/input_context.h", r"get_raw_input", "raw input entry the injector feeds"),
    ("Input inject", "src/input_context.h", r"handle_input", "action dispatch after raw input"),
    ("Entities", "src/character.h", r"get_overlay_ids", "Character overlay list in game order"),
    ("Entities", "src/monster.h", r"get_overlay_ids", "monster overlay list in game order"),
    ("Entities", "src/creature.h", r"enum class FacingDirection", "facing: NONE/LEFT/RIGHT only"),
    ("Vehicles", "src/vehicle.h", r"bub_part_pos", "game-side absolute part position (pre-rotated mounts)"),
    ("Vehicles", "src/vehicle.h", r"tileray face", "continuous facing angle"),
    ("Vehicles", "src/vehicle.cpp", r"multiples of 15", "the 15-degree pathing clamp (NOT an orientation quantum)"),
    ("Tiles draw order", "src/cata_tiles.cpp", r"drawing_layers|multitile_keys", "11 draw layers / 8 multitile keys"),
    ("Animation", "src/animation.cpp", r"void.*animation|tilecontext|draw_.*animation", "animation dispatch (Phase 8 effects hook)"),
    ("Main loop", "src/main.cpp", r"int main", "process entry; --frontend=ipc arg parsing lands near here"),
]

def main():
    results = []
    for area, rel, pat, why in HOOKS:
        path = os.path.join(CDDA, rel)
        entry = {"area": area, "file": rel, "pattern": pat, "why": why,
                 "found": False, "line": None, "text": None}
        if os.path.exists(path):
            rx = re.compile(pat)
            for i, line in enumerate(open(path, encoding="utf-8", errors="replace"), 1):
                if rx.search(line):
                    entry.update(found=True, line=i, text=line.strip()[:200])
                    break
        results.append(entry)
        print(("FOUND " if entry["found"] else "MISS  ") +
              f"{rel}:{entry['line']} [{area}] {entry['text'] or pat}")
    outdir = os.path.join(ROOT, "tools", "phase0", "results")
    os.makedirs(outdir, exist_ok=True)
    with open(os.path.join(outdir, "hooks.json"), "w") as f:
        json.dump(results, f, indent=2)
    md = ["# Hook-point catalogue (Phase 0)", "",
          "Extracted mechanically from CDDA `0.I-1` (release tag 0.I-1) @ `7b2efa5cea38e4d4d97dd0e63b28b9148623da59`",
          "by `tools/phase0/extract_hooks.py`. A MISS means the pattern did not",
          "match at that file/line shape — see docs/recon.md for the resolution.", ""]
    cur = None
    for e in results:
        if e["area"] != cur:
            cur = e["area"]; md += [f"## {cur}", ""]
        loc = f"`{e['file']}:{e['line']}`" if e["found"] else f"`{e['file']}` — **NOT FOUND**"
        md.append(f"- {loc} — {e['why']}")
        if e["found"]: md.append(f"  - `{e['text']}`")
    md.append("")
    with open(os.path.join(ROOT, "docs", "hook-points.md"), "w") as f:
        f.write("\n".join(md))
    n = sum(1 for e in results if e["found"])
    print(f"\n{n}/{len(results)} hook points found")

if __name__ == "__main__":
    main()
