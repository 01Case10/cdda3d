# Vehicle Stage-G build — plan

Status: STARTED 2026-11-24 (Joey's go, after the slope-normals
plan closed at §15). Design settled in the 2026-11-24 prototype
thread: mockups v1–v3, profile approved ("profile and shape look
good"), tex6 colouring rules, engine captures in 98e2a0b. The
prototype (`~/workspace/vehicle-mockup/mockup3.py` +
`export_models.py`, out of repo) is the SPEC: this build ports it
into the engine so any vehicle composed from parts gets the
treatment at scene prep, not just the gallery car.

## 1. What Stage G is for vehicles

The closed catalog is the PARTS (462 vpart defs), not vehicles.
Per-role constructors turn a part's sprite into fine geometry;
a vehicle-level analysis pass over the part layout supplies the
zones the constructors need (cabin, deck line, front/rear); the
vehicle is composed at scene-prep time from the wire's per-part
tile/state/facing, so a modified vehicle is just a different
part list. Parts with no constructor keep today's coarse
textured stamp (option (b), 98e2a0b).

## 2. Architecture

- `vehicles::analysis` — VehicleModel + vpart names →
  `VehicleLayout` in the vehicle's own frame. Local tile coords
  ARE the parts' mounts (the wire's `mount` is the pre-rotation
  slot; `abs_pos` is its world placement). Zones: roof tiles
  (any part with the wire ROOF flag), glass tiles (windshield),
  cabin x-range (roof ∪ glass), roof front edge, vehicle
  front/rear tile x, row extent. Roles per part from the base
  id: roof / windshield / wheel / frame / seat / body panel /
  internal (engine_, tank, dashboard, horn_, wheel_mount_,
  vehicle_alarm, integrated_heater_small — no sprite, no
  constructor).
- `voxelizer::vehicle` — constructors + profile + colouring,
  sprite-injected like `props::model_for` (caller supplies
  base id → sprite). Output: one `PropModel` per occupied tile
  in vehicle frame, size [32, 96, 32] (mockup z × 2).
- `engine::scene_prep` — builds layouts for the frame's
  vehicles, resolves sprites from the tileset materials (the
  material's top-layer image, the same image the prototype
  read), converts to `PropModelData`, rotates by the vehicle's
  cardinal facing (derived from abs vs mount), stamps into the
  FineLayer at each tile's abs position. A tile with a composed
  model suppresses its coarse stamps; a vehicle tile with no
  constructed part keeps the coarse textured stamp.

## 3. The composition spec (from the prototype, mockup units;

48 per z-level in the prototype, 96 in the engine — ×2)

- CLEAR 9 (body floor), FRAME_Z (3, 11), DECK 26 (beltline),
  GLASS_TOP 40, ROOF_Z (40, 45), WHEEL_R 16, GLASS (70, 92, 110).
- Deck heightfield `deck_top_at`: front tile lerps DECK→17
  (hood leading edge), rear tile lerps 21→30, the tile ahead of
  the roof's rear edge lerps DECK→33 (rear-glass ramp zone),
  else DECK.
- Roof parts: sprite alpha extruded 40→top; rear edge of the
  rearmost roof tile slopes 41→45 over its first 8 columns;
  inset 4 from edges facing non-roof tiles. Crown + cabin
  enclosure (exporter): roof top = 45 − round(3·d²), d = lateral
  distance from the vehicle's lateral centre over its width;
  roof extended across the full cabin cross-section (door rows)
  at the roof tiles' x, plus a 4-column strip past the front
  roof edge where no roof segment exists.
- Windshield: per sprite pixel, body panel CLEAR→z1−3 in sprite
  colour, glass sheet z1−3→z1 in GLASS, z1 lerps GLASS_TOP at
  the cabin edge → DECK at the front edge.
- Frame/xlframe: alpha extruded 3→11. Seats: CLEAR+2→20.
  Other body panels: alpha extruded CLEAR→deck_top_at.
- Rear glass: on the rear tile's front half, glass sheet
  z1−3→z1, z1 lerps 30→41.
- Side glass + pillars on cabin edge rows (tiles in the cabin
  x-range that aren't glass tiles): outer row glass DECK→32,
  next two rows glass DECK→GLASS_TOP (tumblehome); the 4
  columns at the cabin's rear end are solid pillars
  (24, 26, 30) DECK→GLASS_TOP instead.
- Wheel constructor (per tile carrying a wheel part): disc at
  the tile's centre column, outer 3 rows on the vehicle's outer
  side, radius 16 around z=16; tyre colour = sprite top-centre
  pixel, hub = sprite centre pixel (|dx| ≤ 5).

## 4. The colouring spec (tex6 + one fix)

Per part, from its sprite: the 12 most frequent colours
(cleaning table), the dominant colour, the trim colour (most
frequent colour with luminance < 95 among the top 40, else
dominant × 0.55). Per fine row of a segment, by depth below
the segment top: depth < 1 → cleaned sprite colour; panels at
depth < 2 → trim; below → dominant with a downward gradient
(→ ×0.7 across the span, ×0.85 at the bottom unit). Glass is
uniform GLASS; wheels and pillars keep their constructor
colours. Body-panel SIDES unify: SIDE_PAINT = mean(door
dominant, halfboard dominant), SIDE_TRIM = door's most common
non-dominant colour — panel sides use those instead of their
own tables. Underbody: any non-wheel voxel below mockup
z 8.5 is shaded ×0.3 (shadow mass).

THE FIX (new vs the prototype — the hood squiggle, root-caused
2026-11-24): panel TOPS also unify. The prototype computed a
vehicle PAINT (median of the achromatic panel dominants) and a
`repaint()` shift (colour + PAINT − part dominant) but never
applied it; hood tops stayed per-part cleaned sprite colours
and jogged at tile seams. S2 applies repaint to the top rows
of achromatic panel parts (chromatic parts — the blue roof —
keep their own colours). Acceptance is measured: the composed
car's hood-tile top colours collapse to one paint family.

## 5. Slices

- S1: analysis + geometry composition for the car layout,
  flat placeholder colours. Gate: per-tile cell occupancy
  equals the prototype's `car_models.bin` exactly (the
  prototype is the spec).
- S2: the colouring spec incl. the top unification. Gate:
  colour assertions (glass uniform, roof chromatic-preserved,
  panel sides = SIDE_PAINT family, hood tops unified).
- S3: scene_prep wiring — capture path first (the profile
  capture switches from the prototype .bin to the engine
  composer; acceptance diff vs the §15 capture = hood tops
  only), then the frame path; coarse fallback intact.
- S4: pickup (cargo bed constructor: border walls + hollow
  floor) and bicycle (frame rails + the wheel constructor at
  full tile). Other vehicles fall back per tile until their
  roles are covered.
- S5: captures of all three, two-pass review per the prop
  rules; Joey's sign-off is acceptance.

## 6. Known limits (stated up front)

- Cardinal facings exact; diagonal facings compose in mount
  space and rotate by quarter turns, so a diagonal vehicle
  renders as its cardinal approximation (the coarse path has
  the same tile-aligned nature; the wire's continuous facing
  still drives interpolation).
- Per-frame re-stamp cost for moving vehicles is unmeasured
  (same open item as the fine props path generally).
- `#variant` sprites remain unreachable (the wire carries base
  ids only) — constructors see base sprites, as the prototype
  did.

## 7. S1 outcome (2026-11-24)

Done. `voxelizer::vehicle` holds the layout types, the zone
analysis and the composer (the engine adapts `VehicleModel`
to the layout in S3 — the voxelizer cannot depend on the
vehicles crate, and the vehicles crate's rule stands: models
are stamped at the parts' exported absolute positions; the
layout only describes the vehicle's internal arrangement).

The gate (`vehicle_stage_g_gate`) composes the real `car`
def with the real sprites and matches the prototype's
exported models on all 24 tiles — occupied-cell count and
FNV-1a hash of the sorted occupied indices, per tile.
Sensitivity verified: a 1-unit deck change (26 → 25) fails
the gate with per-tile drift; restored, it passes.
Voxelizer suite 116/116 undisturbed. Colours in S1 are the
segments' raw colours; S2 replaces them with the §4 spec.

## 8. S2 outcome (2026-11-24)

Done. `voxelizer::vehicle::Colorizer` ports the tex6 rules
(per-part tables from the sprites, cleaned tops, trim row,
side unification onto SIDE_PAINT/SIDE_TRIM, downward
gradient, underbody ×0.3) and applies the prototype's
unapplied `repaint()` to panel tops. Two porting traps
recorded: Python's `min` takes the FIRST of equal minima
(Rust's `min_by_key` takes the last — the clean() lookup
iterates by hand), and `round()` is half-even throughout.

Gates: `vehicle_color_gate` (voxelizer, synthetic sprites,
hand-computed values) 6/6 — paint median, cross-part top
unification with deltas preserved, chromatic exclusion,
side paint/trim, glass/wheel bypass, underbody. Engine
gate gains `car_colours_follow_tex6_plus_top_unification`:
the real car's PAINT is (41,41,41); 66% of achromatic
front-top voxels sit exactly on it (the hood mass — it sat
on halfboard's (153,153,153) before the fix); strongly
blue tops hold at ~47% (the roof is untouched). Failing-
first verified: with repaint disabled the on-paint share
is 0/4006 and the gate fails. S1 occupancy gate still
green — colouring does not move a single cell.

## 9. S3 outcome (2026-11-24) — wiring

`scene_prep::vehicle_fine_scene` composes every fully-named
vehicle in a VehicleScene into fine PropModels
(VEHICLE_PROP_ID_BASE 1,000,000, assistant-chosen) and
returns the covered tile set; all three fine paths (raster
unlit, raster lit, RT props) merge the result into the prop
scene and stamp coarse vehicles with the covered tiles
excluded (`vehicles::stamp_vehicle_mapped_except`). A
vehicle with ANY unnamed part stays fully coarse — partial
composition would mix two looks on one vehicle. Sprite
resolution reads the MaterialTable's top layer PNGs from the
textures dir; rotation is derived from mount deltas
(exact match, else least-wrong by dot product). Coarse
paths unchanged; voxelizer is now a real engine dependency.

Acceptance: the profile capture renders through the engine
composer (VehicleScene in, no prototype .bin). Quad count
identical to the prototype path (221,840). Diff vs the S15
capture: 0.73% of pixels (raster and RT alike), all inside
the car's bounding box, every sampled transition the old
per-part grey family moving onto the vehicle paint
(41,41,41) — glass, wheels, sides and the blue roof are
pixel-identical. Engine suite green (exit 0), vehicles +
voxelizer 130/130.

## 10. S4 outcome (2026-11-24) — pickup bed, bicycle, variants

Bed: Zones now computes bed_tiles (cargo_space tiles with
no roof part, flooded over 4-connected trunk tiles). On bed
tiles, cargo/trunk panels extrude to BED_FLOOR (14) inside
and keep deck height on a 4-wide rim facing any non-bed
tile — the bed reads hollow with walls and a tailgate. The
rear-glass pass is gated on the rear tile actually backing
a cabin (it previously painted a glass sheet over the
pickup's tailgate tiles).

Thin vehicles (single tile row, e.g. the bicycle): a bike
is a profile object, not a footprint. Footprint parts
extrude only in the centre band (rows 13..19); wheel
sprites are true side profiles and are mapped sprite-row →
height in rows 15..18, drawn after the tile's other parts
so the wheel wins the shared plane; generic parts cap at
20; "saddle" joins the seat branch; basketsm gets the
container rule (rim 26 / floor 17). First render had the
rear wheel swallowed by the basket's solid extrusion —
the deferred wheel draw + hollow basket fixed it.

State variants: NOT built. Open-door sprites exist in the
tileset (vp_door_full_left_open, vp_door_front_open_rot),
but choosing one needs the door's variant family, which
the wire does not carry — the same gap as the #variant
limitation from the (b) work. Named here, unscheduled.

Gates: vehicle_s4_gate 2/2 (bed floor/rim heights, thin
band + profile-wheel occupancy); the S1/S2 car gate is
unchanged and green — the car has no bed tiles and is not
thin, by construction.

## 11. S5 bike correction (2026-11-24, Joey's diagnosis)

Joey: the bicycle sprite is from a side view while the car
sprites are top down. Verified against the tileset: EVERY
bicycle-specific sprite is a side elevation (wheel_bicycle,
vp_saddle — handlebar curve over stem over saddle,
vp_foot_pedals — a crank, vp_basketsm_bike_rear), while the
generic vp_basketsm is top-down and xlframe does not
resolve in the material table at all (the bike's frame was
never drawn). The S4 band extrusions were therefore wrong
in kind, not degree.

New thin rule: on a thin vehicle every part is drawn as a
side elevation in the centre plane (draw_profile: sprite
rows -> height, rows 15..18), wheels deferred so they draw
over the parts behind them, and the tileset's bike variant
vp_<base>_bike_rear wins over the generic sprite when it
exists. Thin itself is now single-row AND no bed tiles —
a one-row cargo cart keeps the footprint treatment (its
sprites are top-down); the first cut profile-mapped the
S4 gate's one-row bed layout and the gate caught it.

## 12. Sprite-resolution fix (2026-11-24, Joey: "that still does not look like a bike")

Root cause, measured: the profile mechanism was fine; the
WHEELS were drawn from the wrong sprite. The bicycle
wheel's real art is rotation-keyed (vp_wheel_bicycle_rotE/
N/S/W — side elevation vs edge-on viewpoints); no material
exists under vp_wheel_bicycle (the part's tileset entries
are the multitile vp_wheel_bicycle_front/_rear, whose fg
is the rotation list). The composer's chain missed them
and landed on bare `wheel_bicycle` — a 20x20 item icon
from small.png whose alpha sits in the frame's top-left
(15px of margin beneath it), which the linear profile map
turned into a small wheel floating ~14 units up.

Audit of the 37 gallery vparts (old chain): most resolve
to their vp_ material; suffixed-variant art exists for
frame (vp_frame_horizontal), seat (vp_seat_rear),
halfboard, door_trunk (already hand-pinned); bare-keyed
entries are genuine for frame and seat; alternator_car/
truck, battery_car, stereo and horn_bicycle resolved to
their item sprites (under-hood parts; prototype parity,
left as-is); xlframe and tank resolve to nothing (tank is
internal by design; xlframe's art exists only as
connection-suffixed sprites with no entry reaching the
composer — the remaining frame gap on car and bike).

Fix, three layers:
- Composer (voxelizer::vehicle::resolve_part_sprite, now
  public): order is vp_<base>, then for wheel parts
  vp_<base>_front/_rear, then the fallback pins, then the
  bare name LAST (it is genuine for frame/seat but an item
  icon for wheels — order is the protection).
- Build (voxelizer): materials gain `sprite_layer` — for
  WHEEL entries whose fg is a rotation list, a dedicated
  layer holding the _rotE element (the canonical side
  elevation); every other entry's sprite_layer equals its
  top_layer. Scoped to wheels on purpose: other parts'
  rotation variants are same-projection redraws whose
  primary the composition is built on (an unscoped rule
  measurably drifted the car's doors/halfboards).
- Engine: Material parses sprite_layer (falling back to
  top_layer in older assets); vehicle_fine_scene's
  resolver reads it.

Gate: vehicle_sprite_gate — the wheel resolves to
vp_wheel_bicycle_front, its art is a grounded disc at
least 20px wide and tall, the composed wheel touches the
ground across at least 20 columns, and no wheel part
resolves to a bare name. Failing-first verified (icon:
"wheel art width 4px" via the _front entry's edge-on
primary). Car gate byte-identical throughout — the car
was never touched by the fix. The 5.1 golden gained only
sprite_layer fields (= top_layer on the fixture set).

Note: regenerating target/gallery_assets (required —
they were built before this change) also refreshed
scene materials from the current tileset source; the
committed car captures were NOT re-rendered in this
commit, so capture-level background drift vs the stale
assets is expected and is not vehicle drift (the car
models are pinned exactly by the S1 gate).

## 13. S7 synthesized frame tubes (2026-11-24, Joey: "Synthisise the tubes")

The approved invention from the S5 options, closing the
xlframe gap: on thin vehicles, compose_vehicle adds a
diamond frame in the centre plane after the part pass —
rear hub -> seat top, rear hub -> bottom bracket,
bracket -> seat top, bracket -> head, seat top -> head
(top tube), head -> front hub. Anchors are measured from
the resolved sprites' alpha bboxes (hub = wheel bbox
centre, bracket = pedals bbox centre, falling back to
the hub midpoint), so the tubes plug into the art as
drawn; seat top / head sit over the bracket at
TUBE_TOP_Z 25 (bracket -7 / +9 in x), inside the saddle
sprite's mass. Tube colour = the saddle/pedals sprite's
most common dark colour (the bike's reads brown off the
saddle art), tagged "pillar" so the colourizer passes it
through. Rasterization paints every integer column in
the run — a parameter-stepped diagonal skipped a column
at fractional endpoints (gate-caught).

Gate: vehicle_tube_gate 3/3 (chainstay gap column, top
tube, centre-plane containment), failing-first verified
for both fill assertions. Car and S4/S6 gates unchanged
and green — tubes are thin-only. With S6's wheel fix the
bicycle now renders as one connected object: two real
wheels, the saddle/pedals/basket slices, and the frame
tying them together.
