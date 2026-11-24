//! Visibility recompute (slice S3): LOS, apparent light,
//! outside, memory inputs — replacing the S1/S2 placeholders
//! (z-level lit/seen rule, enclosure-flood outside).
//!
//! The model ports the game's own pipeline as far as the sim's
//! grid supports it, with each simplification named:
//!
//! - **Opacity.** The game's per-tile transparency cache derives
//!   from `map_data_common_t::transparent`, which `set_flag`
//!   (0.I-1 mapdata.cpp:847) sets iff the def carries TRANSPARENT
//!   or TRANSLUCENT. The registry carries the authored flag
//!   strings, so the sim rule is exact, not a proxy: a tile is
//!   opaque iff its terrain def lacks both flags OR its furniture
//!   def (when present) lacks both (either opaque layer zeroes
//!   the combined transparency, matching the cache semantics).
//!   A missing def reads as opaque (fail closed). Fields are
//!   treated as transparent (smoke's real opacity is a field
//!   property the def table does not carry numerically — S5 may
//!   revisit). Per-tile transparency *coefficients* are not in
//!   the def data either, so every transparent tile uses
//!   LIGHT_TRANSPARENCY_OPEN_AIR (2.3/60) — the game's own
//!   coefficient for air — for flood/light attenuation.
//! - **LOS.** A flood (plan §3.2's own word) from the viewpoint
//!   over transparent tiles: a tile is reached when the
//!   accumulated transparency cost along the flood path keeps
//!   exp(-cost) > 0.1 — the `obstructed` cutoff of
//!   apparent_light_helper (0.I-1 lightmap.cpp:725,
//!   vis <= LIGHT_TRANSPARENCY_SOLID + 0.1) — and its Euclidean
//!   distance is <= 60 (MAX_VIEW_DISTANCE; at 0.0383/tile the
//!   cost cutoff and the range agree, by the header's own
//!   design comment). Opaque tiles adjacent to a reached tile
//!   are visible endpoints (castLight marks the first opaque
//!   cell of an arc). Vertical: the tile above a reached tile is
//!   visible as an endpoint (you see the ceiling/roof over a
//!   visible floor), and the flood continues upward through
//!   NO_FLOOR tiles; downward, the flood enters the level below
//!   only through NO_FLOOR / GOES_DOWN tiles. With no player
//!   (stub_house) there is no culling: the fixture snapshot is
//!   fully observed and tiles classify by light alone.
//! - **Ambient light.** Per `build_sunlight_cache` (0.I-1
//!   lightmap.cpp:310): sky-exposed tiles take
//!   `natural_light_level`; other tiles take a directional sun
//!   cast (toward the sun's azimuth, attenuated by the same
//!   exp(-cost) form the game uses for propagation) floored at
//!   `inside_light_level` = (z >= 0 && outside light >
//!   LIGHT_SOURCE_BRIGHT) ? LIGHT_AMBIENT_DIM*0.8 = 4.0 :
//!   LIGHT_AMBIENT_LOW = 3.5. Simplifications: the cast is
//!   top-down 2D (sun altitude scales the incident value via
//!   sun_light_at but does not slant the cast — the game's own
//!   cache is likewise top-down per level); moonlight has no
//!   upstream position function, so it reaches only sky-exposed
//!   tiles directly and interiors get it only via the inside
//!   floor. z < 0 tiles take the inside floor (the game's
//!   fully-inside fill).
//! - **Scene light sources** (`[[light]]` extras) propagate like
//!   `apply_light_source`: luminance attenuates exp(-cost)
//!   through transparent tiles, opaque neighbours receive the
//!   reaching tile's value (their lit face), the source tile
//!   gets sm = luminance, and colour accumulates into a
//!   light-colour-cache analogue (source colour * attenuated
//!   luminance). Same z-level only.
//! - **Opaque tiles' light.** The game's apparent_light_helper
//!   evaluates an opaque tile from the quadrants of its visible
//!   non-opaque neighbours (its lit faces). The sim's
//!   approximation: an opaque tile's lm is raised to the best
//!   transparent neighbour's lm attenuated by one tile of air —
//!   an exterior wall face reads ~the yard's light, an interior
//!   wall reads the room's.
//! - **Collapse.** `apparent_light_at` (0.I-1 lightmap.cpp:801)
//!   verbatim in structure: visible tiles use apparent = lm
//!   (vis = 1 for flood-reached tiles). BRIGHT when apparent >
//!   LIGHT_SOURCE_BRIGHT (10) or sm > 0; then LIT when apparent
//!   > LIGHT_AMBIENT_LIT (10) — upstream both constants are
//!   10.0, so that branch is dead upstream too and is kept for
//!   fidelity; LOW when apparent >= vision_threshold; else
//!   BLANK. Not visible: BRIGHT_ONLY when the tile carries a
//!   light source (sm > 0) within range (the game's
//!   sense-a-source-through-obstruction case), else BLANK.
//!   DARK and MEMORIZED are never produced here: upstream
//!   apparent_light_at never returns them (DARK comes from
//!   map::light_at, which the exporter does not use; MEMORIZED
//!   is stamped by the memory path in world.rs).
//! - **seen** follows the exporter's quantisation
//!   (voxel_real_adapter.cpp): BLANK -> 0, anything else -> 255
//!   (MEMORIZED -> 64 applies to live tiles, which are never
//!   MEMORIZED; memory blocks carry seen 0 — see world.rs).
//! - **frac** (Addendum 11): BRIGHT -> 255; LOW/LIT ->
//!   round(255 * light_above_threshold(apparent, vision_threshold))
//!   (master smooth_lighting.cpp:30); else 0.
//! - **tint** (Addendum 11): round(255 * illumination_hue(cc,
//!   lm)) per channel (master smooth_lighting.cpp:229 +
//!   tile_tint.cpp compute_tile_tint, tint_max_alpha = 80);
//!   uncoloured light yields white [255, 255, 255].
//! - **outside** = is_outside && !is_roofed, resolved from the
//!   grid: is_outside ports `build_outside_cache` (0.I-1
//!   map.cpp) — z < 0 never; otherwise a tile is NOT outside
//!   when any INDOORS-flagged terrain/furniture tile is within
//!   Chebyshev distance 1 on its level (the game's 3x3 stamp;
//!   stamps from tiles outside the scene extent are not
//!   modelled). is_roofed ports the fork's backported
//!   `map::is_roofed` (fork/hooks/backport_patches.md B2):
//!   walk z upward; roofed when a level has a floor — terrain
//!   that is not NO_FLOOR and not GOES_DOWN, or TRANSPARENT_FLOOR
//!   / NO_FLOOR_WATER terrain — or SUN_ROOF_ABOVE furniture one
//!   level below a checked level. Vehicle parts (VPFLAG_ROOF)
//!   are skipped: sim vehicles are wire part lists, not grid
//!   tiles.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

use proto::LitLevel;

use crate::defs::{DefRegistry, Kind};
use crate::environment::{
    self, WeatherState, LIGHT_AMBIENT_DIM, LIGHT_AMBIENT_LIT, LIGHT_AMBIENT_LOW,
    LIGHT_SOURCE_BRIGHT, LIGHT_TRANSPARENCY_OPEN_AIR,
};
use crate::scenes::{LightSpec, SUBMAP};
use crate::world::{BlockState, TILES};

/// Per-block computed wire arrays, parallel to `World::blocks`.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockVis {
    pub lit: Vec<LitLevel>,
    pub seen: Vec<u8>,
    pub frac: Vec<u8>,
    /// Interleaved RGB, 3 per tile.
    pub tint: Vec<u8>,
    pub outside: Vec<bool>,
}

/// The whole recompute result: per-block arrays plus the set of
/// currently-visible tile positions (the memory update input).
#[derive(Debug)]
pub struct VisResult {
    pub blocks: Vec<BlockVis>,
    pub visible: HashSet<[i32; 3]>,
}

#[derive(Debug, Clone, Copy, Default)]
struct Props {
    transparent: bool,
    indoors: bool,
    no_floor: bool,
    goes_down: bool,
    transparent_floor: bool,
    no_floor_water: bool,
    sun_roof_above: bool,
}

fn has_flag(reg: &DefRegistry, kind: Kind, id: u32, flag: &str) -> bool {
    id != 0
        && reg
            .def_by_id(kind, id)
            .is_some_and(|d| d.flags.iter().any(|f| f == flag))
}

fn layer_transparent(reg: &DefRegistry, kind: Kind, id: u32) -> bool {
    if id == 0 {
        return true; // no layer present
    }
    match reg.def_by_id(kind, id) {
        // mapdata.cpp set_flag: transparent iff TRANSPARENT or TRANSLUCENT.
        Some(d) => d
            .flags
            .iter()
            .any(|f| f == "TRANSPARENT" || f == "TRANSLUCENT"),
        None => false, // unknown def: fail closed (opaque)
    }
}

fn props_of(reg: &DefRegistry, terrain: u32, furniture: u32) -> Props {
    let flag = |kind: Kind, id: u32, f: &str| has_flag(reg, kind, id, f);
    Props {
        transparent: layer_transparent(reg, Kind::Terrain, terrain)
            && layer_transparent(reg, Kind::Furniture, furniture),
        indoors: flag(Kind::Terrain, terrain, "INDOORS")
            || flag(Kind::Furniture, furniture, "INDOORS"),
        no_floor: flag(Kind::Terrain, terrain, "NO_FLOOR"),
        goes_down: flag(Kind::Terrain, terrain, "GOES_DOWN"),
        transparent_floor: flag(Kind::Terrain, terrain, "TRANSPARENT_FLOOR"),
        no_floor_water: flag(Kind::Terrain, terrain, "NO_FLOOR_WATER"),
        sun_roof_above: flag(Kind::Furniture, furniture, "SUN_ROOF_ABOVE"),
    }
}

/// Chebyshev-1 INDOORS stamp (build_outside_cache) + the
/// is_roofed upward walk, per tile.
fn compute_outside(props: &HashMap<[i32; 3], Props>, zs: &[i32]) -> HashMap<[i32; 3], bool> {
    let max_z = zs.iter().copied().max().unwrap_or(0);
    let mut out = HashMap::new();
    for (&pos, _) in props.iter() {
        let (x, y, z) = (pos[0], pos[1], pos[2]);
        if z < 0 {
            out.insert(pos, false);
            continue;
        }
        let mut is_outside = true;
        'stamp: for dy in -1..=1 {
            for dx in -1..=1 {
                if let Some(p) = props.get(&[x + dx, y + dy, z]) {
                    if p.indoors {
                        is_outside = false;
                        break 'stamp;
                    }
                }
            }
        }
        let mut roofed = false;
        let mut cz = z + 1;
        while cz <= max_z {
            // SUN_ROOF_ABOVE furniture one level below the
            // checked level roofs the tile (backport B2 checks
            // the furniture at check_z - 1, which for the first
            // step is the tile itself).
            if props.get(&[x, y, cz - 1]).is_some_and(|p| p.sun_roof_above) {
                roofed = true;
                break;
            }
            match props.get(&[x, y, cz]) {
                None => break, // level not modelled above: no roof
                Some(p) => {
                    if p.transparent_floor || p.no_floor_water {
                        roofed = true;
                        break;
                    }
                    if !p.no_floor && !p.goes_down {
                        roofed = true;
                        break;
                    }
                }
            }
            cz += 1;
        }
        out.insert(pos, is_outside && !roofed);
    }
    out
}

/// Directional sun cast: step from `pos` toward the sun
/// (azimuth from South, west-positive -> in sim coords, +y is
/// south: to-sun = (-sin az, +cos az)) in half-tile steps.
/// Returns the exp(-cost) attenuation if the ray leaves the
/// scene extent unobstructed, or 0.0 when an opaque tile blocks
/// it. The starting tile itself is not counted (its own light
/// is what is being computed).
fn sun_cast_factor(props: &HashMap<[i32; 3], Props>, pos: [i32; 3], azimuth_deg: f64) -> f32 {
    let az = azimuth_deg.to_radians();
    let dir = (-az.sin(), az.cos()); // (dx, dy), y south-positive
    if dir.0.abs() < 1e-9 && dir.1.abs() < 1e-9 {
        return 1.0;
    }
    let mut cost = 0.0f32;
    let mut last: Option<[i32; 3]> = None;
    let mut step = 0.5f64;
    loop {
        let x = pos[0] as f64 + dir.0 * step;
        let y = pos[1] as f64 + dir.1 * step;
        let tile = [x.floor() as i32, y.floor() as i32, pos[2]];
        if tile != pos && Some(tile) != last {
            match props.get(&tile) {
                None => return (-cost).exp(), // left the modelled extent
                Some(p) => {
                    if !p.transparent {
                        return 0.0;
                    }
                    cost += LIGHT_TRANSPARENCY_OPEN_AIR;
                    if cost > 60.0 * LIGHT_TRANSPARENCY_OPEN_AIR {
                        return (-cost).exp();
                    }
                }
            }
            last = Some(tile);
        }
        step += 0.5;
        if step > 64.0 {
            return (-cost).exp();
        }
    }
}

/// Dijkstra flood over transparent tiles from `start` (same
/// z), returning tile -> accumulated transparency cost. Step
/// cost is LIGHT_TRANSPARENCY_OPEN_AIR times the geometric step
/// length (1 cardinal, sqrt(2) diagonal), so cost tracks
/// distance the way castLight's row propagation does. The flood
/// stops at the obstructed cutoff: exp(-cost) <= 0.1, i.e.
/// cost >= ln(10) — at 0.0383/tile that is exactly
/// MAX_VIEW_DISTANCE (60) of open air.
/// Total-ordered f32 wrapper for the Dijkstra heap.
#[derive(Debug, Clone, Copy, PartialEq)]
struct CostKey(f32);
impl Eq for CostKey {}
impl PartialOrd for CostKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for CostKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

fn flood(props: &HashMap<[i32; 3], Props>, start: [i32; 3]) -> HashMap<[i32; 3], f32> {
    let max_cost = 10.0f32.ln();
    let mut cost: HashMap<[i32; 3], f32> = HashMap::new();
    let mut heap: BinaryHeap<(Reverse<CostKey>, [i32; 3])> = BinaryHeap::new();
    cost.insert(start, 0.0);
    heap.push((Reverse(CostKey(0.0)), start));
    while let Some((Reverse(CostKey(c)), p)) = heap.pop() {
        if c > cost[&p] {
            continue;
        }
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let n = [p[0] + dx, p[1] + dy, p[2]];
                let Some(np) = props.get(&n) else { continue };
                if !np.transparent {
                    continue;
                }
                let step = LIGHT_TRANSPARENCY_OPEN_AIR * ((dx * dx + dy * dy) as f32).sqrt();
                let nc = c + step;
                if nc >= max_cost {
                    continue;
                }
                if nc < *cost.get(&n).unwrap_or(&f32::INFINITY) {
                    cost.insert(n, nc);
                    heap.push((Reverse(CostKey(nc)), n));
                }
            }
        }
    }
    cost
}

/// The visible set: flood-reached tiles, their opaque endpoint
/// neighbours, and the z-crossing rules (module docs).
fn compute_visible(props: &HashMap<[i32; 3], Props>, viewpoint: [i32; 3]) -> HashSet<[i32; 3]> {
    let mut visible: HashSet<[i32; 3]> = HashSet::new();
    // reached[z] = transparent tiles the flood reached on z.
    let mut reached: HashMap<i32, HashSet<[i32; 3]>> = HashMap::new();
    let pz = viewpoint[2];
    let base_set: HashSet<[i32; 3]> = flood(props, viewpoint).keys().copied().collect();
    reached.insert(pz, base_set.clone());

    // Expand to neighbouring z levels, one direction at a time:
    // from each reached tile, the tile one level over is an
    // endpoint (visible); if it is transparent it also seeds
    // that level's flood. Upward, any reached tile shows what is
    // above it; downward, the base tile must be an opening
    // (NO_FLOOR / GOES_DOWN).
    for dz in [1, -1] {
        let mut prev = base_set.clone();
        let mut level = pz + dz;
        loop {
            let mut seeds: HashSet<[i32; 3]> = HashSet::new();
            for p in &prev {
                let over = [p[0], p[1], level];
                let Some(op) = props.get(&over) else {
                    continue;
                };
                let base_ok = dz > 0 || props.get(p).is_some_and(|bp| bp.no_floor || bp.goes_down);
                if !base_ok {
                    continue;
                }
                if op.transparent {
                    seeds.insert(over);
                } else {
                    visible.insert(over); // endpoint only
                }
            }
            if seeds.is_empty() {
                break;
            }
            // Multi-source flood: flood from each seed and union
            // (scene levels are small; correctness first).
            let mut level_reached: HashSet<[i32; 3]> = HashSet::new();
            for s in &seeds {
                for t in flood(props, *s).keys() {
                    level_reached.insert(*t);
                }
            }
            reached.insert(level, level_reached.clone());
            prev = level_reached;
            level += dz;
        }
    }

    for set in reached.values() {
        for p in set {
            visible.insert(*p);
            // Opaque endpoint neighbours.
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let n = [p[0] + dx, p[1] + dy, p[2]];
                    if props.contains_key(&n) {
                        visible.insert(n);
                    }
                }
            }
        }
    }
    visible.insert(viewpoint);
    // Range rule: Euclidean distance <= MAX_VIEW_DISTANCE (60).
    visible.retain(|p| {
        let d = ((p[0] - viewpoint[0]) as f32).powi(2)
            + ((p[1] - viewpoint[1]) as f32).powi(2)
            + ((p[2] - viewpoint[2]) as f32).powi(2);
        d.sqrt() <= 60.0
    });
    visible
}

/// compute_tile_tint + illumination_hue (master tile_tint.cpp /
/// smooth_lighting.cpp:229), returning wire ubytes (round(255 *
/// hue)); white when the colour cache is uncoloured.
fn tile_tint(cc: [f32; 3], scalar_light: f32) -> [u8; 3] {
    const WHITE: [u8; 3] = [255, 255, 255];
    // light_color_rgb::is_colored (lightmap.h:68).
    if !(cc[0] > 0.0 || cc[1] > 0.0 || cc[2] > 0.0) {
        return WHITE;
    }
    let min_ch = cc[0].min(cc[1]).min(cc[2]);
    let sat = [cc[0] - min_ch, cc[1] - min_ch, cc[2] - min_ch];
    let sat_mag = sat[0].max(sat[1]).max(sat[2]);
    if sat_mag < 0.01 {
        return WHITE;
    }
    let ratio = if scalar_light > 0.1 {
        (sat_mag / scalar_light).min(1.0)
    } else {
        0.0
    };
    let alpha = (ratio * 80.0) as u8; // tint_max_alpha = 80, truncating cast
    if alpha == 0 {
        return WHITE;
    }
    let tint = [
        (sat[0] / sat_mag * 255.0) as u8,
        (sat[1] / sat_mag * 255.0) as u8,
        (sat[2] / sat_mag * 255.0) as u8,
    ];
    let colored = cc[0].max(cc[1]).max(cc[2]);
    let t =
        ((colored - LIGHT_AMBIENT_LOW) / (LIGHT_AMBIENT_LIT - LIGHT_AMBIENT_LOW)).clamp(0.0, 1.0);
    let fade = t * t * (3.0 - 2.0 * t);
    let s = fade * (alpha as f32 / 80.0).min(1.0);
    let mut out = [0u8; 3];
    for c in 0..3 {
        let hue = 1.0 + s * (tint[c] as f32 / 255.0 - 1.0);
        out[c] = (hue * 255.0).round().clamp(0.0, 255.0) as u8;
    }
    out
}

/// Run the full recompute over the world's blocks.
#[allow(clippy::too_many_arguments)]
pub fn compute(
    reg: &DefRegistry,
    blocks: &[BlockState],
    lights: &[LightSpec],
    viewpoint: Option<[i32; 3]>,
    spectator_origin: [i32; 3],
    turn: i64,
    weather: &WeatherState,
) -> VisResult {
    // Index every tile.
    let mut props: HashMap<[i32; 3], Props> = HashMap::new();
    let mut zs: Vec<i32> = Vec::new();
    for b in blocks {
        if !zs.contains(&b.origin.z) {
            zs.push(b.origin.z);
        }
        for (i, t) in b.tiles.iter().enumerate() {
            let pos = [
                b.origin.x + (i % SUBMAP) as i32,
                b.origin.y + (i / SUBMAP) as i32,
                b.origin.z,
            ];
            props.insert(pos, props_of(reg, t.terrain, t.furniture));
        }
    }

    let outside_map = compute_outside(&props, &zs);

    // Ambient terms.
    let natural0 = environment::natural_light_level(weather, turn, 0);
    let sun_part = environment::incident_sunlight(weather, turn);
    let (azimuth, _altitude) = environment::sun_azimuth_altitude(turn);
    let inside_level = |z: i32| -> f32 {
        // build_sunlight_cache: (zlev >= 0 && outside_light >
        // LIGHT_SOURCE_BRIGHT) ? DIM*0.8 : LOW.
        if z >= 0 && natural0 > LIGHT_SOURCE_BRIGHT {
            LIGHT_AMBIENT_DIM * 0.8
        } else {
            LIGHT_AMBIENT_LOW
        }
    };

    // lm + colour cache + sm per tile.
    let mut lm: HashMap<[i32; 3], f32> = HashMap::new();
    let mut cc: HashMap<[i32; 3], [f32; 3]> = HashMap::new();
    let mut sm: HashMap<[i32; 3], f32> = HashMap::new();
    for (&pos, _) in props.iter() {
        let direct = if pos[2] < 0 {
            0.0
        } else if outside_map[&pos] {
            environment::natural_light_level(weather, turn, pos[2]).max(0.0)
        } else if sun_part > 0.0 {
            sun_part * sun_cast_factor(&props, pos, azimuth)
        } else {
            0.0
        };
        let base = if outside_map[&pos] && pos[2] >= 0 {
            direct
        } else {
            direct.max(inside_level(pos[2]))
        };
        lm.insert(pos, base);
        cc.insert(pos, [0.0; 3]);
    }
    for light in lights {
        if !props.contains_key(&light.pos) {
            continue;
        }
        let reached = flood(&props, light.pos);
        for (pos, cost) in &reached {
            let value = light.intensity * (-cost).exp();
            *lm.get_mut(pos).expect("lm covers props") += value;
            let c = cc.get_mut(pos).expect("cc covers props");
            for ch in 0..3 {
                c[ch] += light.color[ch] * value;
            }
        }
        // Opaque neighbours of reached tiles: their lit face
        // receives the reaching tile's attenuated value.
        let mut faces: HashMap<[i32; 3], f32> = HashMap::new();
        for (pos, cost) in &reached {
            let att = (-cost).exp();
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let n = [pos[0] + dx, pos[1] + dy, pos[2]];
                    if !reached.contains_key(&n) && props.contains_key(&n) {
                        let e = faces.entry(n).or_insert(0.0);
                        *e = e.max(light.intensity * att);
                    }
                }
            }
        }
        for (pos, value) in faces {
            *lm.get_mut(&pos).expect("lm covers props") += value;
            let c = cc.get_mut(&pos).expect("cc covers props");
            for ch in 0..3 {
                c[ch] += light.color[ch] * value;
            }
        }
        let e = sm.entry(light.pos).or_insert(0.0);
        *e = e.max(light.intensity);
    }

    // Opaque tiles take their lit faces' light: raise each
    // opaque tile's lm to the best transparent neighbour's lm
    // attenuated by one tile of air (module docs). Computed
    // from a snapshot so the pass is order-independent.
    {
        let before = lm.clone();
        for (&pos, p) in props.iter() {
            if p.transparent {
                continue;
            }
            let mut best = lm[&pos];
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let n = [pos[0] + dx, pos[1] + dy, pos[2]];
                    if let (Some(np), Some(v)) = (props.get(&n), before.get(&n)) {
                        if np.transparent {
                            best = best.max(v * (-LIGHT_TRANSPARENCY_OPEN_AIR).exp());
                        }
                    }
                }
            }
            lm.insert(pos, best);
        }
    }

    // Visibility.
    let visible = match viewpoint {
        Some(vp) => compute_visible(&props, vp),
        // Spectator: everything is observed; the threshold is
        // taken at the view origin.
        None => props.keys().copied().collect(),
    };
    let ambient_at_view = lm
        .get(&viewpoint.unwrap_or(spectator_origin))
        .copied()
        .unwrap_or(0.0);
    let vt = environment::vision_threshold(ambient_at_view);

    // Collapse per block.
    let mut out_blocks: Vec<BlockVis> = Vec::with_capacity(blocks.len());
    for b in blocks {
        let mut bv = BlockVis {
            lit: vec![LitLevel::BLANK; TILES],
            seen: vec![0; TILES],
            frac: vec![0; TILES],
            tint: vec![0; TILES * 3],
            outside: vec![false; TILES],
        };
        for i in 0..TILES {
            let pos = [
                b.origin.x + (i % SUBMAP) as i32,
                b.origin.y + (i / SUBMAP) as i32,
                b.origin.z,
            ];
            let apparent = lm.get(&pos).copied().unwrap_or(0.0);
            let self_light = sm.get(&pos).copied().unwrap_or(0.0);
            let is_vis = visible.contains(&pos);
            let lit = if is_vis {
                if apparent > LIGHT_SOURCE_BRIGHT || self_light > 0.0 {
                    LitLevel::BRIGHT
                } else if apparent > LIGHT_AMBIENT_LIT {
                    // Dead upstream too (both constants are
                    // 10.0); kept verbatim for fidelity.
                    LitLevel::LIT
                } else if apparent >= vt {
                    LitLevel::LOW
                } else {
                    LitLevel::BLANK
                }
            } else if self_light > 0.0 {
                LitLevel::BRIGHT_ONLY
            } else {
                LitLevel::BLANK
            };
            bv.lit[i] = lit;
            bv.seen[i] = if lit == LitLevel::BLANK { 0 } else { 255 };
            bv.frac[i] = match lit {
                LitLevel::BRIGHT => 255,
                LitLevel::LOW | LitLevel::LIT => {
                    let f = environment::light_above_threshold(apparent, vt);
                    (f * 255.0).round().clamp(0.0, 255.0) as u8
                }
                _ => 0,
            };
            let tint = tile_tint(cc.get(&pos).copied().unwrap_or([0.0; 3]), apparent);
            bv.tint[i * 3] = tint[0];
            bv.tint[i * 3 + 1] = tint[1];
            bv.tint[i * 3 + 2] = tint[2];
            bv.outside[i] = outside_map.get(&pos).copied().unwrap_or(false);
        }
        out_blocks.push(bv);
    }

    VisResult {
        blocks: out_blocks,
        visible,
    }
}
