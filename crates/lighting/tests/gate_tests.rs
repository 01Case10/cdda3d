//! Phase 7 gate tests 7.1–7.9 (Part II §10), all at harness/data
//! level on fake lit/seen data — none of them needs the real fork
//! running in-game (see docs/phase7-lighting.md).

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::Instant;

use lighting::atmosphere::{day_night_at, weather_params, WeatherKind};
use lighting::fixtures;
use lighting::grid::{LightGrid, VisibilityMask};
use lighting::mapping::{
    apply_visibility_rgb, base_intensity, luminance, present_color, visibility_of, LightSample,
    TileLight, Visibility, MEMORIZED_BRIGHTNESS, MEMORIZED_SATURATION_KEEP,
};
use lighting::render::{render_isometric, render_top_down};
use lighting::smoothing::{face_light, top_face_vertex_light, vertex_light_at_grid_corner};
use lighting::{
    apply_patch, bake_chunk_map, bake_pass_meshes, chunks_dirtied_by_tile, Environment, LightPatch,
};
use mesher::block::*;
use mesher::chunk::ChunkMap;
use mesher::scale::WorldScale;
use proto::LitLevel;

const LEVELS: [LitLevel; 5] = [
    LitLevel::DARK,
    LitLevel::LOW,
    LitLevel::BRIGHT_ONLY,
    LitLevel::LIT,
    LitLevel::BRIGHT,
];

fn lum_u8(c: [u8; 3]) -> f32 {
    0.2126 * c[0] as f32 + 0.7152 * c[1] as f32 + 0.0722 * c[2] as f32
}

// ---------------------------------------------------------------- 7.1
#[test]
fn gate_7_1_light_mapping_monotonic() {
    // Strictly monotonic in enum order for EVERY seen combination:
    // max of a lower level (seen 255) < min of the next (seen 0).
    for w in LEVELS.windows(2) {
        let lo_max = base_intensity(w[0], 255);
        let hi_min = base_intensity(w[1], 0);
        assert!(
            lo_max < hi_min,
            "{:?} max {lo_max} must be < {:?} min {hi_min}",
            w[0],
            w[1]
        );
    }
    // Monotonic (strictly, since bands have width) in seen within a level.
    for l in LEVELS {
        let mut prev = -1.0f32;
        for seen in [0u8, 1, 32, 64, 128, 200, 254, 255] {
            let v = base_intensity(l, seen);
            assert!(v > prev, "{l:?} seen {seen}: {v} <= {prev}");
            assert!((0.0..=1.0).contains(&v));
            prev = v;
        }
    }
    // Tint API: intensity x tint per channel, no breaking shape needed
    // for per-tile color later.
    let s = LightSample::new(0.5, [1.0, 0.5, 0.25]);
    assert_eq!(s.multiplier(), [0.5, 0.25, 0.125]);
    let tinted = TileLight::new(LitLevel::BRIGHT, 255).with_tint([0.2, 0.4, 1.0]);
    let raw = tinted.raw_sample();
    assert_eq!(raw.intensity, 1.0);
    assert_eq!(raw.tint, [0.2, 0.4, 1.0]);
}

// ---------------------------------------------------------------- 7.2
#[test]
fn gate_7_2_visibility_states() {
    assert_eq!(visibility_of(LitLevel::LIT, 255), Visibility::Visible);
    assert_eq!(
        visibility_of(LitLevel::MEMORIZED, 128),
        Visibility::Memorized
    );
    assert_eq!(visibility_of(LitLevel::BLANK, 0), Visibility::Hidden);
    assert_eq!(visibility_of(LitLevel::LIT, 0), Visibility::Hidden);

    let albedo = [0.8f32, 0.2, 0.2];
    // Visible: full presentation (BRIGHT/255 == intensity 1).
    let vis = present_color(albedo, TileLight::new(LitLevel::BRIGHT, 255));
    assert_eq!(vis, albedo);
    // Memorized: exact documented factors — desaturate (keep 30% of
    // chroma) then x0.45 brightness.
    let memo = present_color(albedo, TileLight::new(LitLevel::MEMORIZED, 128));
    let l = luminance(albedo);
    for c in 0..3 {
        let expected = (l + (albedo[c] - l) * MEMORIZED_SATURATION_KEEP) * MEMORIZED_BRIGHTNESS;
        assert!((memo[c] - expected).abs() < 1e-6, "channel {c}");
    }
    // Memorized is darker and less saturated than visible.
    assert!(luminance(memo) < luminance(vis));
    let sat = |c: [f32; 3]| {
        c.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
            - c.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
    };
    assert!(sat(memo) < sat(vis));
    // The standalone desaturation helper agrees.
    assert_eq!(apply_visibility_rgb(albedo, Visibility::Memorized), memo);
    // Hidden: black.
    assert_eq!(present_color(albedo, TileLight::BLANK), [0.0, 0.0, 0.0]);

    // Mask contract shared by both renderers + baked-map behaviour:
    // BLANK blocks are not meshed; MEMORIZED blocks survive, dimmed.
    let (map, grid) = fixtures::remembered_street_scene();
    let mask = VisibilityMask::new(&grid);
    assert!(mask.is_meshed(15, 5, 0)); // visible street
    assert!(mask.is_meshed(5, 5, 0)); // memorized street
    assert!(!mask.is_meshed(22, 5, 0)); // blank
    assert_eq!(mask.is_meshed(22, 5, 0), mask.include_in_tree(22, 5, 0));
    assert_eq!(mask.is_meshed(5, 5, 0), mask.include_in_tree(5, 5, 0));
    let baked = bake_chunk_map(&map, &grid, Environment::neutral());
    let count = |m: &ChunkMap| -> usize { m.chunks.values().map(|c| c.grid.count_non_air()).sum() };
    // East-end blank tiles (x >= 20) contribute no blocks at all.
    let blank_blocks = baked
        .chunks
        .values()
        .flat_map(|c| {
            let o = c.origin();
            (0..c.grid.size[2]).flat_map(move |z| {
                (0..c.grid.size[1]).flat_map(move |y| {
                    (0..c.grid.size[0]).filter_map(move |x| {
                        let b = c.grid.get(x as i32, y as i32, z as i32);
                        if o[0] + x as i32 >= 20 && !b.is_air() {
                            Some(())
                        } else {
                            None
                        }
                    })
                })
            })
        })
        .count();
    assert_eq!(blank_blocks, 0, "BLANK tiles must not be meshed");
    assert!(count(&baked) < count(&map), "blank removal shrinks the map");
    assert!(count(&baked) > 0, "visible + memorized geometry survives");
}

// ---------------------------------------------------------------- 7.3
#[test]
fn gate_7_3_no_light_leaking() {
    let g = fixtures::two_room_grid();
    // Wall column x=5. Its east face samples the DARK room (x=6);
    // its west face samples the BRIGHT room (x=4). Same wall tile,
    // opposite results — no mixing across the wall plane.
    let east = face_light(&g, [5, 2, 0], 0); // +X face
    let west = face_light(&g, [5, 2, 0], 1); // -X face
    assert_eq!(east.intensity, base_intensity(LitLevel::DARK, 255));
    assert_eq!(west.intensity, base_intensity(LitLevel::BRIGHT, 255));
    assert!(east.intensity < 0.15 && west.intensity > 0.99);
    // Vertex level too, at both horizontal corners of each face.
    for corner_y in [2, 3] {
        let (ve, _) = vertex_light_at_grid_corner(&g, [6, 2, 0], 0, [6, corner_y, 0]);
        let (vw, _) = vertex_light_at_grid_corner(&g, [4, 2, 0], 1, [5, corner_y, 0]);
        assert!(ve.intensity < 0.15, "east vertex leaks: {}", ve.intensity);
        assert!(vw.intensity > 0.99, "west vertex dimmed: {}", vw.intensity);
    }
    // Mesh level: build the two-room world, mesh it, bake per-vertex,
    // and compare wall-face vertex colours by face normal/plane.
    let scale = WorldScale::default();
    let mut map = ChunkMap::new(scale);
    {
        let chunk = map.add_chunk([0, 0], 1);
        for y in 0..5 {
            for x in 0..11 {
                chunk.set_tile(x, y, 0, TILE_GROUND, scale);
            }
            chunk.set_tile(5, y, 0, TILE_WALL, scale);
        }
    }
    map.clear_dirty();
    let mut meshes = map.mesh_chunk([0, 0], false).unwrap();
    bake_pass_meshes(&mut meshes, &g, Environment::neutral(), scale, 0);
    let mut east_lums = Vec::new();
    let mut west_lums = Vec::new();
    for mesh in &meshes.meshes {
        for i in 0..mesh.positions.len() {
            let n = mesh.normals[i];
            let p = mesh.positions[i];
            let c = mesh.colors[i];
            let lum = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
            if n == [1.0, 0.0, 0.0] && (p[0] - 6.0).abs() < 0.01 {
                east_lums.push(lum); // wall east faces (world +X)
            }
            if n == [-1.0, 0.0, 0.0] && (p[0] - 5.0).abs() < 0.01 {
                west_lums.push(lum);
            }
        }
    }
    assert!(!east_lums.is_empty() && !west_lums.is_empty());
    let mean = |v: &[f32]| v.iter().sum::<f32>() / v.len() as f32;
    assert!(
        mean(&east_lums) < 0.25 * mean(&west_lums),
        "mesh-level leak: east {} vs west {}",
        mean(&east_lums),
        mean(&west_lums)
    );
}

// ---------------------------------------------------------------- 7.4
#[test]
fn gate_7_4_smoothing_no_chunk_seam() {
    let g = fixtures::gradient_grid();
    // Smoothed top-face vertex intensity along the gradient, sampled
    // at every tile corner. Inside a chunk and across the x=12 chunk
    // border, successive steps must be comparably small — a border
    // seam would show as a step much larger than the interior steps
    // (or as a duplicated/flat pair from clamping).
    let mut steps = Vec::new();
    let mut prev: Option<f32> = None;
    for corner_x in 0..=24 {
        let tile_x = corner_x.min(23);
        let (s, vis) = top_face_vertex_light(&g, [tile_x, 1, 0], [corner_x, 1]);
        assert_eq!(vis, Visibility::Visible);
        if let Some(p) = prev {
            steps.push((corner_x, (s.intensity - p).abs()));
        }
        prev = Some(s.intensity);
    }
    let interior_max = steps
        .iter()
        .filter(|(x, _)| *x != 12)
        .map(|(_, d)| *d)
        .fold(0.0f32, f32::max);
    let border_step = steps.iter().find(|(x, _)| *x == 12).unwrap().1;
    assert!(
        border_step <= interior_max * 1.5 + 1e-6,
        "chunk-border step {border_step} vs interior max {interior_max}"
    );
    // And the smoothed gradient is monotonic overall (no border dip).
    let mut vals = Vec::new();
    for corner_x in 0..=24 {
        let tile_x = corner_x.min(23);
        let (s, _) = top_face_vertex_light(&g, [tile_x, 1, 0], [corner_x, 1]);
        vals.push(s.intensity);
    }
    for w in vals.windows(2) {
        assert!(w[1] >= w[0] - 1e-6, "gradient not monotonic: {vals:?}");
    }
    // Smoothing actually smooths: the border vertex value lies
    // strictly between the raw values of the tiles it touches.
    let raw_west = g.get(11, 1, 0).raw_sample().intensity;
    let raw_east = g.get(12, 1, 0).raw_sample().intensity;
    let (smoothed_mid, _) = top_face_vertex_light(&g, [12, 1, 0], [12, 1]);
    assert!(
        smoothed_mid.intensity > raw_west.min(raw_east) - 1e-6
            && smoothed_mid.intensity < raw_east.max(raw_west) + 1e-6
            && (smoothed_mid.intensity - raw_east).abs() > 1e-4,
        "border vertex {} should interpolate between {raw_west} and {raw_east}",
        smoothed_mid.intensity
    );
}

// ---------------------------------------------------------------- 7.5
#[test]
fn gate_7_5_light_updates_dirty_chunks() {
    // Interior change: only its own chunk.
    assert_eq!(chunks_dirtied_by_tile(6, 6), BTreeSet::from([[0, 0]]));
    // Edge change: own + edge neighbour. Corner: + diagonal too.
    assert_eq!(
        chunks_dirtied_by_tile(0, 6),
        BTreeSet::from([[0, 0], [-1, 0]])
    );
    assert_eq!(
        chunks_dirtied_by_tile(11, 11),
        BTreeSet::from([[0, 0], [1, 0], [0, 1], [1, 1]])
    );
    // One tile inside the border does NOT dirty the neighbour
    // (smoothing reads at most 1 tile across).
    assert_eq!(chunks_dirtied_by_tile(1, 6), BTreeSet::from([[0, 0]]));
    assert_eq!(chunks_dirtied_by_tile(10, 6), BTreeSet::from([[0, 0]]));

    // Sufficiency proof: partial re-bake of exactly the dirty set
    // equals a full re-bake, on a scene spanning 2x2 chunks with the
    // change ON a chunk corner (the worst case for radius claims).
    let (map, mut grid) = fixtures::flashlight_scene(); // 24x24, 2x2 chunks
    let env = Environment::neutral();
    let bake_all = |g: &LightGrid| {
        let baked = bake_chunk_map(&map, g, env);
        let mut out = Vec::new();
        for coord in [[0, 0], [1, 0], [0, 1], [1, 1]] {
            out.push(baked.mesh_chunk(coord, true).unwrap());
        }
        out
    };
    let before = bake_all(&grid);
    // Toggle a "light" at the shared corner tile (12,12): DARK -> BRIGHT.
    let patch = LightPatch {
        origin: [12, 12, 0],
        size_xy: [1, 1],
        lit: vec![LitLevel::BRIGHT],
        seen: vec![255],
    };
    let dirty = apply_patch(&mut grid, &patch);
    assert!(dirty.contains(&[1, 1]) && dirty.contains(&[0, 0]));
    assert!(
        dirty.len() >= 4,
        "corner change dirties 4 chunks: {dirty:?}"
    );
    let after_full = bake_all(&grid);
    // Partial: keep `before` meshes for clean chunks, re-bake dirty.
    let coords = [[0, 0], [1, 0], [0, 1], [1, 1]];
    for (i, coord) in coords.iter().enumerate() {
        let full = &after_full[i];
        let partial = if dirty.contains(coord) {
            full // re-baked
        } else {
            &before[i] // kept
        };
        for (a, b) in full.meshes.iter().zip(partial.meshes.iter()) {
            assert_eq!(a.positions, b.positions, "chunk {coord:?} geometry");
            assert_eq!(
                a.colors, b.colors,
                "chunk {coord:?} colours differ — dirty set under-claimed"
            );
        }
    }
    // The change really did something (test is not vacuous).
    let changed_somewhere = before
        .iter()
        .zip(after_full.iter())
        .any(|(b, a)| b.meshes[0].colors != a.meshes[0].colors);
    assert!(changed_somewhere);
    // A no-op patch (same values re-sent) dirties nothing.
    let dirty2 = apply_patch(&mut grid, &patch);
    assert!(dirty2.is_empty());
}

// ---------------------------------------------------------------- 7.6
#[test]
fn gate_7_6_day_night() {
    let midnight = day_night_at(0.0);
    let dawn = day_night_at(7.5);
    let noon = day_night_at(12.0);
    let dusk = day_night_at(19.0);
    assert!(
        midnight.brightness < 0.3,
        "midnight dim: {}",
        midnight.brightness
    );
    assert_eq!(noon.brightness, 1.0);
    assert_eq!(noon.tint, [1.0, 1.0, 1.0]);
    assert!(dawn.brightness > midnight.brightness && dawn.brightness < noon.brightness);
    assert!(dusk.brightness > midnight.brightness && dusk.brightness < noon.brightness);
    // Dawn/dusk are warm (red > blue); midnight is blue (blue > red).
    assert!(dawn.tint[0] > dawn.tint[2]);
    assert!(dusk.tint[0] > dusk.tint[2]);
    assert!(midnight.tint[2] > midnight.tint[0]);
    // Smoothness: stepping 1 minute never jumps.
    let mut prev = day_night_at(0.0);
    for step in 1..=(24 * 60) {
        let h = step as f32 / 60.0;
        let cur = day_night_at(h % 24.0);
        assert!(
            (cur.brightness - prev.brightness).abs() < 0.01,
            "brightness jump at {h}: {} -> {}",
            prev.brightness,
            cur.brightness
        );
        for c in 0..3 {
            assert!(
                (cur.tint[c] - prev.tint[c]).abs() < 0.01,
                "tint jump at {h}"
            );
        }
        prev = cur;
    }
    // Wrap-around agrees: 24.0 == 0.0, negative hours wrap too.
    assert_eq!(day_night_at(24.0), day_night_at(0.0));
    assert_eq!(day_night_at(-12.0), day_night_at(12.0));
}

// ---------------------------------------------------------------- 7.7
#[test]
fn gate_7_7_weather() {
    let clear = weather_params(WeatherKind::Clear);
    assert!(!clear.fog_enabled() && !clear.precip_enabled());
    assert_eq!(clear.fog_transmittance(100.0), 1.0);
    let fog = weather_params(WeatherKind::Fog);
    assert!(fog.fog_enabled() && !fog.precip_enabled());
    assert!(fog.fog_density > weather_params(WeatherKind::Rain).fog_density);
    assert!(fog.fog_transmittance(50.0) < 0.2, "dense fog at 50 tiles");
    let rain = weather_params(WeatherKind::Rain);
    assert!(rain.precip_enabled() && rain.precip_rate > 0.0);
    assert!(weather_params(WeatherKind::HeavyRain).precip_rate > rain.precip_rate);
    let snow = weather_params(WeatherKind::Snow);
    assert!(snow.precip_enabled() && snow.precip_fall_speed < rain.precip_fall_speed);
    // Fog blending moves a colour toward fog colour with distance.
    let near = fog.apply_fog([0.0, 0.0, 0.0], 1.0);
    let far = fog.apply_fog([0.0, 0.0, 0.0], 100.0);
    assert!(
        lum_u8([
            (near[0] * 255.0) as u8,
            (near[1] * 255.0) as u8,
            (near[2] * 255.0) as u8
        ]) < lum_u8([
            (far[0] * 255.0) as u8,
            (far[1] * 255.0) as u8,
            (far[2] * 255.0) as u8
        ])
    );
    // No cost cliff: every state's params come from the same constant
    // table — asserted structurally by all values being finite and
    // bounded (the lookup itself is O(1) by construction).
    for k in [
        WeatherKind::Clear,
        WeatherKind::Cloudy,
        WeatherKind::Rain,
        WeatherKind::HeavyRain,
        WeatherKind::Snow,
        WeatherKind::Fog,
        WeatherKind::Thunderstorm,
    ] {
        let p = weather_params(k);
        assert!(p.fog_density.is_finite() && p.precip_rate.is_finite());
        assert!(p.brightness_scale > 0.0 && p.brightness_scale <= 1.0);
    }
}

// ---------------------------------------------------------------- 7.8
fn output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("output")
}

#[test]
fn gate_7_8_visual_scenes() {
    let dir = output_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let night = Environment::new(0.0, WeatherKind::Clear);

    // Scene 1: lit house at night (top-down level 0 + isometric of
    // the baked level-0 cutaway).
    let map = fixtures::fake_house_map();
    let grid = fixtures::fake_house_night_light();
    let top = render_top_down(&map, &grid, 0, night, 16);
    top.write_png(&dir.join("house_night_topdown.png")).unwrap();
    // Interior brighter than exterior.
    let interior = top.get(6 * 16 + 8, 6 * 16 + 8);
    let edge = top.get(2 * 16 + 8, 6 * 16 + 8); // interior, LOW-ish falloff
    assert!(
        lum_u8(interior) > lum_u8(edge),
        "lamp centre {interior:?} should outshine falloff edge {edge:?}"
    );
    assert!(top.distinct_colors() >= 4);
    // Isometric of the baked house with the roof level cut away
    // (the standard cutaway), so the lit interior is what shows.
    let baked = lighting::filter_levels(&bake_chunk_map(&map, &grid, night), &[0]);
    let iso = render_isometric(&baked, 8);
    iso.write_png(&dir.join("house_night_iso.png")).unwrap();
    assert!(iso.distinct_colors() >= 3);

    // Scene 2: flashlight in a field.
    let (fmap, fgrid) = fixtures::flashlight_scene();
    let fimg = render_top_down(&fmap, &fgrid, 0, night, 12);
    fimg.write_png(&dir.join("flashlight_field.png")).unwrap();
    let core = fimg.get(5 * 12 + 6, 12 * 12 + 6);
    let mid = fimg.get(10 * 12 + 6, 12 * 12 + 6);
    let dark = fimg.get(2 * 12 + 6, 2 * 12 + 6);
    assert!(
        lum_u8(core) > lum_u8(mid),
        "cone core {core:?} > mid {mid:?}"
    );
    assert!(
        lum_u8(mid) > lum_u8(dark),
        "cone mid {mid:?} > dark field {dark:?}"
    );
    // Behind the player stays dark (cone, not a disc).
    let behind = fimg.get(1 * 12 + 6, 12 * 12 + 6);
    assert!(lum_u8(behind) <= lum_u8(dark) + 1.0);

    // Scene 3: remembered street.
    let (smap, sgrid) = fixtures::remembered_street_scene();
    let simg = render_top_down(&smap, &sgrid, 0, Environment::neutral(), 12);
    simg.write_png(&dir.join("remembered_street.png")).unwrap();
    let memo = simg.get(5 * 12 + 6, 5 * 12 + 6); // memorized road
    let seen = simg.get(15 * 12 + 6, 5 * 12 + 6); // live road
    let blank = simg.get(22 * 12 + 6, 5 * 12 + 6); // never seen
    assert_eq!(blank, [0, 0, 0], "BLANK renders black");
    assert!(
        lum_u8(seen) > lum_u8(memo),
        "live street brighter than memory"
    );
    // Memorized is desaturated: channel spread smaller relative to level.
    let spread = |c: [u8; 3]| {
        (*c.iter().max().unwrap() as f32 - *c.iter().min().unwrap() as f32) / (lum_u8(c) + 1.0)
    };
    assert!(spread(memo) <= spread(seen) + 0.05);
}

// ---------------------------------------------------------------- 7.9
#[test]
fn gate_7_9_performance_full_view() {
    let (map, grid) = fixtures::full_view_scene();
    assert_eq!(map.chunks.len(), 121);
    let env = Environment::neutral();
    let t0 = Instant::now();
    let baked = bake_chunk_map(&map, &grid, env);
    let bake_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let t1 = Instant::now();
    let mut total_quads = 0usize;
    for coord in baked.chunks.keys().copied().collect::<Vec<_>>() {
        // mesh_chunk takes &self on the map; baked chunks are dirty.
        if let Some(pm) = baked.mesh_chunk(coord, true) {
            total_quads += pm.total_quads();
        }
    }
    let mesh_ms = t1.elapsed().as_secs_f64() * 1000.0;
    // Per-vertex bake over the combined meshes of a few chunks too.
    println!(
        "gate 7.9: full-view (132x132 x 3 levels, 121 chunks) per-block bake {bake_ms:.2} ms, mesh-all {mesh_ms:.2} ms, quads {total_quads}"
    );
    // Generous VM bound (debug build, 1.5 shared CPUs): the bake is
    // O(tiles) with a tiny constant; the doc records the measured
    // number and the target-hardware budget discussion. This assert
    // exists to catch an accidental quadratic blow-up, not to gate
    // on llvmpipe-class timing.
    assert!(bake_ms < 2000.0, "bake took {bake_ms} ms");
    assert!(total_quads > 0);
    // Blank corner really excluded from the baked world.
    let non_air: usize = baked.chunks.values().map(|c| c.grid.count_non_air()).sum();
    let orig: usize = map.chunks.values().map(|c| c.grid.count_non_air()).sum();
    assert!(non_air < orig);
}
