# sim-defs — full CDDA def importer (Phase 11, slice S1a)

Generates the sim-server's `TileDefs` table from the **real** CDDA
JSON data at the pinned commit (Joey's Option C decision,
2026-11-23: full real import, everything), mirroring the semantics
of the fork's real adapter
(`fork/src/voxel_ipc/voxel_real_adapter.cpp`: `tile_defs()`,
`hot_flags_of()`, `item_wire_id()`).

## Usage

```sh
python3 tools/sim-defs/sim_defs.py census    # -> generated/census.json
python3 tools/sim-defs/sim_defs.py build     # -> generated/defs.json
                                             #    + generated/build_report.json
python3 tools/sim-defs/sim_defs.py validate  # -> generated/validation.json
                                             #    exit 1 on any violation
```

Python 3, stdlib only. Paths default to this repo's
`fork-src/cdda/data/json` and `fork-src/cdda/src/mapdata.h`;
override with `--data` / `--mapdata` / `--out-dir`.

Entries are found by their `"type"` field across **all** folders
under `data/json` (terrain/furniture live in
`furniture_and_terrain/`, monsters in `monsters/`, items in
`items/`, vehicle parts in `vehicleparts/`, traps in `traps.json`,
field types in `field_type.json` + `fields/`, and strays elsewhere).
Files that fail to parse or are not top-level lists are counted in
the census report, never silently skipped (at the pinned commit:
3,353 files, 0 parse failures, 0 non-list files).

## What build emits

`generated/defs.json`: `{protocol_version, source, connect_groups,
defs}` where each def is one wire `TileDef`
(`schema/cdda_ipc.fbs`): `{id, string_id, name, kind, flags,
hot_flags, connect_groups, connects_to_groups, rotate_to_groups,
open_result, close_result, bash_result, looks_like, roof_result,
fg_ref, bg_ref}`. Kinds: Terrain=0, Furniture=1, Trap=2, Field=3,
Monster=4, Item=5, VehiclePart=6.

- `hot_flags` (terrain/furniture only): the `ter_furn_flag` enum
  order is **parsed from `fork-src/cdda/src/mapdata.h`** (139 flags
  at the pinned commit), never hardcoded. JSON flag `"TRANSPARENT"`
  maps to `TFLAG_TRANSPARENT`. Only enum indices < 64 set bits —
  the real adapter truncates the same way; later flags stay in the
  string `flags` list. Flag strings with no enum entry are counted
  and listed in `build_report.json` (16 distinct at the pinned
  commit, e.g. `RUG`, `NO_DECONSTRUCT`, `BARRICADABLE_WINDOW`,
  and one apparent upstream typo, `EASY_NO_DECONSTRUCTDECONSTRUCT`
  — these are flags the game's ter/furn loader does not resolve
  into `ter_furn_flag` either).
- Connect groups (terrain/furniture only): JSON
  `connect_groups`/`connects_to`/`rotates_to` (string or list of
  names) are mapped to indices by the load order of
  `data/json/connect_groups.json` (40 groups). Unknown names are
  counted/listed in the build report (0 at the pinned commit).
- Transforms: `open`/`close` are plain string fields;
  `bash_result` is `bash.ter_set` (terrain) / `bash.furn_set`
  (furniture); `roof_result` is `roof` (terrain only);
  `looks_like` is taken for every kind.
- Names: strings pass through; translation dicts take `str`
  (then `str_sp`/`str_pl`); field types take their first
  intensity level's name (the adapter's `field_type::name(0)`).
  Entries with no name at all fall back to their string_id
  (56 traps, 226 vehicle parts at the pinned commit — their JSON
  genuinely carries no name).

## Documented deviations from the real adapter

1. **Per-kind ids are assigned, not the game's.** The adapter
   emits each kind's own game int ids, which are assigned by the
   C++ loader in load order and are *not reproducible from the
   JSON alone* (they depend on mod/load sequencing inside the
   running game). The sim assigns deterministically: string_ids
   sorted within a kind, ids 1..N, 0 = none. (The game's item ids
   happen to use exactly this sorted scheme already — see
   `item_wire_id()` in the adapter.) Ids collide across kinds by
   design, exactly as in the game; the tile field gives context.
2. **fg_ref / bg_ref are -1 for every def in this slice.** The
   join against the composed tileset `tile_config.json` (the
   tileset crate's fg_refs table) is a follow-up slice. -1 is the
   schema's "none" sentinel and is what the real adapter emits
   when its refs table is absent — no indices are invented.

## Inheritance rule implemented

`copy-from` chains are resolved per JSON type. A parent is looked
up by its `id`, or by its `abstract` value for abstract templates.
The merged entry starts from the parent's *resolved* dict; the
child overrides field-by-field, except that when both parent and
child values for a key are dicts they merge **one level deep**
(child subkeys win — e.g. a child can override just
`bash.ter_set`). Lists and scalars replace wholesale. An `extend`
dict is applied afterwards the CDDA way (lists append, dicts
update, anything else sets). `relative`/`proportional` are
numeric load-time adjustments to fields this importer does not
emit; they are counted in the census and otherwise ignored.
Load directives never survive into a resolved entry.

Entries with an `abstract` key are templates and are **dropped**
(never emitted); entries with no `id` are dropped. At the pinned
commit the two sets are identical — every abstract entry carries
`"abstract"` instead of `"id"` — so validation subtracts the
**union** of the two sets from the census, never both (a literal
"census − abstracts − id-less" would double-subtract).

## Validation and the known-dangling policy

`validate` recomputes the census from source (it does not trust
the build), then checks: per-kind counts match, zero abstracts in
the output, ids and string_ids unique within a kind, every def
has a string_id, the connect-group registry matches
`connect_groups.json` load order, the spot checks pass
(`t_door_c` → open `t_door_o` + roof `t_wood_roof`; glass doors
bash to `t_mdoor_frame`; `t_floor` → roof `t_wood_treated_roof`),
and **zero dangling transform strings**: every
open/close/bash/roof target must resolve within terrain (for
terrain defs) or furniture (for furniture defs), and every
`looks_like` within its own kind.

The pinned upstream data genuinely contains dangling references,
so they are handled explicitly, not silently passed:

- 668 findings at the pinned commit, all inspected and recorded
  in `generated/known_dangling.json` with a per-entry
  classification. Validation fails on any dangling reference
  **not** in that allowlist, and also fails if an allowlist entry
  stops dangling (stale allowlist).
- The findings fall into three inspected classes:
  1. **239 × `t_null`/`f_null`** — the game's built-in null id
     (`mapdata.cpp`: `set_ter_ids()` binds `ter_id("t_null")` as
     the null terrain sentinel; bashing to it means "remove").
     It has no JSON def by design.
  2. **228 × cross-kind `looks_like`** — string ids are global in
     the game and `looks_like` is a sprite fallback, so terrain
     legitimately looks like furniture (`t_gas_pump` →
     `f_machinery_heavy`), vehicle parts look like furniture,
     etc. They only "dangle" under this slice's stricter
     same-kind rule.
  3. **201 × target nowhere in the data** — legacy/removed ids
     (`t_machinery_electronic`, `t_plut_generator`, `t_foam_roof`
     as a roof), tileset-only sprite names used as monster
     `looks_like` (`chud`, `mon_frog`), ids that survive only as
     MIGRATION entries (`fridge`, `armor_lightplate`), and one
     whitespace `looks_like` (`" "`, `t_deck_coating_no_roof_*`).

Regenerate the allowlist only after inspecting new findings:
`validate --update-known-dangling` rewrites it from the current
findings. Do not run it blindly to make a failure go away.
