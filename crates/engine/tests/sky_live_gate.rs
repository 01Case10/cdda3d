//! Phase 10 slice 10.6 gates — live seating.
//!
//! What this file pins:
//! - The wire `outside[]` feed (the game's `is_outside &&
//!   !is_roofed`, §3.4 option (a)) is the sky-exposure source
//!   for BOTH renderers' live paths: the raster factor via
//!   `frame::SkyExposure::from_grid`, and the RT per-cell
//!   `sky_exposed` via the grid the `LightingScene` already
//!   carries. Wave 2's RT interim feed — an engine-side column
//!   scan over the geometry (§3.4 option (b)) — disagrees with
//!   the game on perimeter wall tiles (scan: sheltered by the
//!   roof above; game: exposed), and 10.6 retires it.
//! - The proto `EnvironmentUpdate` conversion seats a
//!   `SkyScene` (the joint the live loop consumes).
//! - The cave factor seating rule for the live loop (§4).
//! - The gate-10.6 visual capture set (§8): both scenes x
//!   {midnight, dawn, noon, dusk} x {Clear, Rain, Fog,
//!   Thunderstorm} x both renderers, written out as contact
//!   sheets for Joey's sign-off (a green run here is NOT the
//!   sign-off — it proves the set exists and is sane).

use engine::frame::{LightingScene, SkyExposure, SkyScene};
use lighting::WeatherKind;

// ---------------------------------------------------------------------
// Raster exposure from the wire grid
// ---------------------------------------------------------------------

#[test]
fn raster_exposure_from_wire_grid() {
    let snap = proto::samples::sample_snapshot();
    let grid = lighting::proto_adapter::grid_from_snapshot(&snap);
    let exposure = SkyExposure::from_grid(&grid);
    // Stub-house rule: interior sheltered, perimeter exposed.
    assert_eq!(exposure.factor([6, 6, 0]), 0.0, "interior floor");
    assert_eq!(exposure.factor([1, 1, 0]), 0.0, "interior corner");
    assert_eq!(exposure.factor([0, 0, 0]), 1.0, "corner wall");
    assert_eq!(exposure.factor([0, 6, 0]), 1.0, "perimeter wall");
    // Tiles the grid does not cover keep the no-data shape:
    // factor 1 (the GI applies globally, as with `sky_exposure:
    // None` — absence never shelters).
    assert_eq!(exposure.factor([40, 40, 0]), 1.0, "outside the grid");
}

// ---------------------------------------------------------------------
// RT exposure follows the wire grid, not the geometry scan
// ---------------------------------------------------------------------

/// Any cell of `world` whose containing game tile is `tile`,
/// as (found, sky_exposed). Tile math mirrors rt.rs's own
/// (block coords, `blocks_per_z_level` blocks per game z).
fn cell_exposure(
    world: &treebuild::world::VoxelWorld,
    tile: [i32; 3],
    min_game_z: i32,
) -> Option<bool> {
    let per = mesher::scale::WorldScale::default().blocks_per_z_level.max(1) as i32;
    world
        .iter()
        .find(|(pos, _)| [pos[0], pos[1], min_game_z + pos[2] / per] == tile)
        .map(|(_, cell)| cell.sky_exposed)
}

#[test]
fn rt_exposure_follows_wire_grid() {
    let lit = LightingScene::fake_house(12.0, WeatherKind::Clear);
    // Both fake-house levels (floor + roof), so the roof
    // level's cells exist to carry the verdict.
    let world = engine::rt::world_for_levels_lit(&[0, 1], &lit);
    // Perimeter wall tile: the game's test says exposed (the
    // wire rule the fixture grid now carries). The retired
    // column scan said sheltered — the roof slab sits above
    // the wall column — so this assertion fails on the scan.
    assert_eq!(
        cell_exposure(&world, [0, 6, 0], lit.min_game_z),
        Some(true),
        "perimeter wall cell takes the wire's exposed verdict"
    );
    // Interior floor tile: both sources agree — sheltered.
    assert_eq!(
        cell_exposure(&world, [6, 6, 0], lit.min_game_z),
        Some(false),
        "interior floor cell is sheltered"
    );
    // Roof tile (z 1 over the interior): exposed.
    assert_eq!(
        cell_exposure(&world, [6, 6, 1], lit.min_game_z),
        Some(true),
        "roof cell is exposed"
    );
}

#[test]
fn rt_exposure_honours_wire_over_geometry() {
    // The converse flip: the game marks the roof tile
    // sheltered (a higher roof the capture's world does not
    // model, a bridge overhead — the game is the truth, the
    // geometry scan cannot know). The wire must win.
    let mut lit = LightingScene::fake_house(12.0, WeatherKind::Clear);
    let mut t = lit.grid.get(6, 6, 1);
    t.outside = false;
    lit.grid.set(6, 6, 1, t);
    let world = engine::rt::world_for_levels_lit(&[0, 1], &lit);
    assert_eq!(
        cell_exposure(&world, [6, 6, 1], lit.min_game_z),
        Some(false),
        "roof cell follows the wire's sheltered verdict"
    );
}

// ---------------------------------------------------------------------
// Fake weather environments + the named sky matrix
// ---------------------------------------------------------------------

#[test]
fn fake_weather_classifies_to_intended_kind() {
    // The 10.2b pattern: a fake environment is only useful if
    // its numerics round-trip through the classifier to the
    // kind it claims to be.
    for (kind, id) in [
        (WeatherKind::Clear, "sunny"),
        (WeatherKind::Rain, "weather_rain"),
        (WeatherKind::Fog, "weather_fog"),
        (WeatherKind::Thunderstorm, "weather_thunderstorm"),
    ] {
        let env = lighting::GameEnvironment::fake_weather(
            43_200.0, 21_600.0, 64_800.0, 1, 4, kind,
        );
        assert_eq!(lighting::classify_weather(&env), kind, "{id} classifies");
        assert_eq!(env.weather_id, id);
        // The seated scene derives the same presentation:
        // cloud cover strictly increases clear < fog < rain <
        // thunderstorm, and the wet kinds precipitate.
        let state = SkyScene::fake_weather(12.0, 4, kind).sky_state();
        let expected_cover = match kind {
            WeatherKind::Clear => 0.05,
            WeatherKind::Fog => 0.70,
            WeatherKind::Rain => 0.80,
            WeatherKind::Thunderstorm => 1.0,
            _ => unreachable!(),
        };
        assert_eq!(state.cloud_cover, expected_cover, "{id} cloud cover");
        assert_eq!(
            state.precip.kind == lighting::PrecipKind::None,
            kind == WeatherKind::Clear || kind == WeatherKind::Fog,
            "{id} precip presence"
        );
    }
}

#[test]
fn named_sky_matrix_resolves() {
    // One resolver for the --sky CLI names and this gate's
    // capture matrix, so the two cannot drift.
    let hours = [
        ("midnight", 0.0),
        ("dawn", 6.5),
        ("noon", 12.0),
        ("dusk", 17.5),
    ];
    let kinds = [
        ("clear", WeatherKind::Clear),
        ("rain", WeatherKind::Rain),
        ("fog", WeatherKind::Fog),
        ("thunderstorm", WeatherKind::Thunderstorm),
    ];
    for (tname, hour) in hours {
        for (wname, kind) in kinds {
            let name = format!("{tname}-{wname}");
            let scene = SkyScene::named(&name)
                .unwrap_or_else(|| panic!("{name} resolves"));
            assert_eq!(
                lighting::classify_weather(&scene.game_env),
                kind,
                "{name} weather"
            );
            assert_eq!(
                scene.game_env.time_of_day_sec,
                hour * 3600.0,
                "{name} hour"
            );
        }
    }
    assert!(SkyScene::named("neutral").is_some());
    assert!(SkyScene::named("noon").is_none(), "bare time is not a name");
    assert!(SkyScene::named("bogus-clear").is_none());
    // The pre-10.6 names keep their exact scenes (capture
    // reproducibility): noon-clear IS fake_day(12, 4).
    assert_eq!(
        SkyScene::named("noon-clear").unwrap().uniforms(),
        SkyScene::fake_day(12.0, 4).uniforms(),
        "noon-clear unchanged"
    );
    assert_eq!(
        SkyScene::named("neutral").unwrap().uniforms(),
        SkyScene::neutral_env().uniforms(),
        "neutral unchanged"
    );
}

// ---------------------------------------------------------------------
// Cave factor seating (§4)
// ---------------------------------------------------------------------

#[test]
fn cave_factor_seats_from_camera_target() {
    // The §4 rule: the dark cave gradient (decision 3C)
    // engages only when the camera target tile is below
    // ground AND the sky does not reach it.
    let mut sheltered = std::collections::HashSet::new();
    sheltered.insert([6, 6, 0]); // roofed surface interior
    sheltered.insert([6, 6, -1]); // cave tile under it
    let exposure = SkyExposure { sheltered };
    // Sheltered surface interior: NOT a cave (the gradient is
    // a cave treatment, not a roof treatment).
    assert_eq!(SkyScene::cave_factor_for(&exposure, [6, 6, 0]), 0.0);
    // Below ground and sheltered: full cave gradient.
    assert_eq!(SkyScene::cave_factor_for(&exposure, [6, 6, -1]), 1.0);
    // Below ground but sky-exposed (a shaft the game marks
    // outside): the surface sky stays.
    assert_eq!(SkyScene::cave_factor_for(&exposure, [6, 6, -2]), 0.0);
    assert_eq!(SkyScene::cave_factor_for(&exposure, [0, 0, -1]), 0.0);
    // The wire-derived set agrees on the stub house's
    // sheltered interior at surface level.
    let snap = proto::samples::sample_snapshot();
    let grid = lighting::proto_adapter::grid_from_snapshot(&snap);
    let wire_exposure = SkyExposure::from_grid(&grid);
    assert_eq!(SkyScene::cave_factor_for(&wire_exposure, [6, 6, 0]), 0.0);
}

// ---------------------------------------------------------------------
// Gate 10.6 visual capture set (§8)
// ---------------------------------------------------------------------

/// GPU helper, the sky_raster_gate convention: a skip is LOUD
/// and proves nothing.
fn gpu() -> Option<(wgpu::Device, wgpu::Queue)> {
    pollster::block_on(async {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await?;
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("sky-live-gate-device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .ok()?;
        Some((device, queue))
    })
}

fn mean_luminance(img: &image::RgbaImage) -> f32 {
    let mut sum = 0f64;
    for px in img.pixels() {
        sum += 0.2126 * px[0] as f64 + 0.7152 * px[1] as f64 + 0.0722 * px[2] as f64;
    }
    (sum / (img.width() * img.height()) as f64) as f32
}

fn distinct_colors(img: &image::RgbaImage) -> usize {
    img.pixels()
        .map(|p| [p[0], p[1], p[2]])
        .collect::<std::collections::HashSet<_>>()
        .len()
}

/// Regression (10.6): an RT frame rendered through
/// `render_frame` — the live loop's entry — must use the
/// seated sky. Before the fix this branch called the no-sky
/// RT entry, so every RT frame rendered under the built-in
/// neutral sky whatever `CaptureArgs::sky` said (the 10.4
/// gates called the sky entry point directly and never saw
/// it): midnight and noon frames came out byte-identical.
#[test]
fn rt_frame_through_render_frame_uses_seated_sky() {
    let Some((device, queue)) = gpu() else {
        eprintln!("rt_frame_through_render_frame_uses_seated_sky: NOT RUN — no wgpu adapter");
        return;
    };
    use engine::capture::{CaptureArgs, RendererKind};
    let render = |sky: SkyScene| {
        let args = CaptureArgs {
            snapshot: "stub-house".to_string(),
            renderer: RendererKind::Rt,
            size: (320, 180),
            camera_target: (6.0, 6.0),
            yaw_deg: 45.0,
            pitch_deg: 45.0,
            distance: 28.0,
            cutaway_z: 1,
            lighting: Some(LightingScene::fake_house(12.0, WeatherKind::Clear)),
            sky: Some(sky),
            ..CaptureArgs::default()
        };
        let outcome = engine::frame::render_frame(&device, &queue, &args).expect("render_frame");
        let (img, _) =
            engine::frame::readback_rgba(&device, &queue, &outcome.texture, outcome.size)
                .expect("readback");
        img
    };
    let midnight = render(SkyScene::named("midnight-clear").unwrap());
    let noon = render(SkyScene::named("noon-clear").unwrap());
    let hash = |img: &image::RgbaImage| {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash_slice(img.as_raw(), &mut h);
        std::hash::Hasher::finish(&h)
    };
    assert_ne!(
        hash(&midnight),
        hash(&noon),
        "RT through render_frame ignores the seated sky (midnight == noon)"
    );
    assert!(
        mean_luminance(&noon) > mean_luminance(&midnight),
        "noon should outbrighten midnight through the live path: {} vs {}",
        mean_luminance(&noon),
        mean_luminance(&midnight)
    );
}

/// The §8 visual set: stub house + sample scene x
/// {midnight, dawn, noon, dusk} x {Clear, Rain, Fog,
/// Thunderstorm} x {raster, RT}, written as PNGs + contact
/// sheets under `target/sky-gate-10.6/` for Joey's sign-off.
/// A green run here is NOT the sign-off — it proves the set
/// exists, is non-blank, and orders sanely (noon-clear
/// brightest of the clears, midnight darkest, noon
/// thunderstorm dimmer than noon clear). The sample scene
/// renders sky-only (no light fixture exists for it — the
/// same configuration its other gates use); the stub house
/// carries the coherent bake (its Phase 7 environment is
/// derived from the resolved sky scene, so bake and sky
/// cannot drift) over the *daytime* light grid: the night
/// fixture's `DARK` exterior capped the building to a black
/// silhouette at every hour and the sheets could not show
/// the GI on the house at all (Joey's review, 2026-11-23).
/// The day grid's lit levels are ceilings, not sources —
/// midnight still renders dark because the sky is dark.
#[test]
fn gate_10_6_visual_capture_set() {
    let Some((device, queue)) = gpu() else {
        eprintln!(
            "================================================================\n\
             gate 10.6 visual set: NOT RUN — no wgpu adapter.\n\
             This run proves NOTHING about the capture set.\n\
             ================================================================"
        );
        return;
    };
    use engine::capture::{CaptureArgs, RendererKind};
    let out_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/sky-gate-10.6");
    std::fs::create_dir_all(&out_dir).expect("create output dir");

    let times = ["midnight", "dawn", "noon", "dusk"];
    let weathers = ["clear", "rain", "fog", "thunderstorm"];
    let mut index = String::from(
        "Gate 10.6 visual capture set (slice 10.6, design §8).\n\
         Contact sheets: rows = midnight, dawn, noon, dusk (top to bottom);\n\
         columns = clear, rain, fog, thunderstorm (left to right).\n\
         Acceptance is Joey's sign-off on the sheets, not this gate.\n",
    );
    for (scene_name, snapshot) in [("house", "stub-house"), ("sample", "sample-scene")] {
        for renderer in [RendererKind::Raster, RendererKind::Rt] {
            let rname = match renderer {
                RendererKind::Raster => "raster",
                RendererKind::Rt => "rt",
            };
            let mut sheet = image::RgbaImage::new(320 * 4, 180 * 4);
            let mut hashes = std::collections::HashMap::new();
            let mut frames: std::collections::HashMap<String, u64> =
                std::collections::HashMap::new();
            let mut clear_lums = std::collections::HashMap::new();
            let mut noon_lums = std::collections::HashMap::new();
            for (ti, tname) in times.iter().enumerate() {
                for (wi, wname) in weathers.iter().enumerate() {
                    let sky_name = format!("{tname}-{wname}");
                    let sky = SkyScene::named(&sky_name).expect("matrix name");
                    let mut args = CaptureArgs {
                        snapshot: snapshot.to_string(),
                        renderer,
                        size: (320, 180),
                        sky: Some(sky.clone()),
                        ..CaptureArgs::default()
                    };
                    if snapshot == "stub-house" {
                        args.camera_target = (6.0, 6.0);
                        args.yaw_deg = 45.0;
                        args.pitch_deg = 45.0;
                        args.distance = 28.0;
                        args.cutaway_z = 1;
                        // Coherent bake: the Phase 7 environment
                        // of the resolved sky scene itself.
                        let hour = sky.game_env.time_of_day_sec / 3600.0;
                        let kind = lighting::classify_weather(&sky.game_env);
                        args.lighting = Some(LightingScene::fake_house_day(hour, kind));
                    } else {
                        args.camera_target = (6.5, 6.0);
                        args.yaw_deg = 45.0;
                        args.pitch_deg = 50.0;
                        args.distance = 15.0;
                        args.cutaway_z = 0;
                    }
                    let outcome = engine::frame::render_frame(&device, &queue, &args)
                        .expect("render_frame");
                    let (img, _) = engine::frame::readback_rgba(
                        &device,
                        &queue,
                        &outcome.texture,
                        outcome.size,
                    )
                    .expect("readback");
                    assert!(
                        distinct_colors(&img) >= 8,
                        "{scene_name}/{rname}/{sky_name}: near-blank frame"
                    );
                    let lum = mean_luminance(&img);
                    if *wname == "clear" {
                        clear_lums.insert(*tname, lum);
                    }
                    if *tname == "noon" {
                        noon_lums.insert(*wname, lum);
                    }
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    std::hash::Hash::hash_slice(img.as_raw(), &mut hasher);
                    let h = std::hash::Hasher::finish(&hasher);
                    hashes.insert(h, sky_name.clone());
                    frames.insert(sky_name.clone(), h);
                    let file = format!("{scene_name}-{rname}-{sky_name}.png");
                    img.save(out_dir.join(&file)).expect("save png");
                    image::imageops::replace(
                        &mut sheet,
                        &img,
                        (wi * 320) as i64,
                        (ti * 180) as i64,
                    );
                }
            }
            // Distinctness: every sky changes the frame. In
            // RT, dawn and dusk exercise the §5.3 overcast
            // collapse: at the fake day's 06:30/17:30 the sun's
            // premultiplied luminance measures ~0.11, under
            // the 0.18 collapse threshold, so the directional
            // term folds into the hemisphere term — dawn and
            // dusk receive the SAME light level, differing
            // only in the sun's azimuth (east vs west), which
            // the collapsed frame does not use for surfaces.
            //
            // Instrument re-scoped 2026-11-23 (Joey's
            // ratification): this pair was originally pinned
            // BYTE-IDENTICAL, which held only because both
            // frames were near-black (~19 mean luminance) and
            // every real difference rounded to the same bytes.
            // The twilight-scatter lift (same session) makes
            // twilight readable (~33), and the sky
            // background's warm glow legitimately sits in the
            // east at dawn and the west at dusk — the frames
            // are mirror images, not copies. The §5.3 claim is
            // the equal light level, so the pin is now:
            // dawn/dusk clear luminance agree within 2%
            // (measured 32.83 vs 32.83 on the house scene).
            if renderer == RendererKind::Rt {
                let (dawn_lum, dusk_lum) = (clear_lums["dawn"], clear_lums["dusk"]);
                let rel = (dawn_lum - dusk_lum).abs() / dawn_lum.max(dusk_lum).max(1e-6);
                assert!(
                    rel < 0.02,
                    "{scene_name}/rt: dawn-clear vs dusk-clear luminance should \
                     agree under the §5.3 collapse (sun luminance ~0.11 < 0.18): \
                     dawn {dawn_lum:.2} dusk {dusk_lum:.2} (rel {rel:.4})"
                );
            }
            {
                // Distinct hash count: 16 for raster; RT is
                // allowed fewer (frozen-weather pairs can still
                // coincide), floor 12.
                if renderer == RendererKind::Rt {
                    assert!(
                        hashes.len() >= 12,
                        "{scene_name}/{rname}: only {} distinct frames of 16",
                        hashes.len()
                    );
                } else {
                    assert_eq!(
                        hashes.len(),
                        16,
                        "{scene_name}/{rname}: expected 16 distinct frames"
                    );
                }
                println!(
                    "10.6 {scene_name}/{rname}: {} distinct frames of 16",
                    hashes.len()
                );
            }
            // Sanity ordering, clear column: noon brightest,
            // midnight darkest.
            assert!(
                clear_lums["noon"] > clear_lums["dawn"]
                    && clear_lums["noon"] > clear_lums["dusk"]
                    && clear_lums["noon"] > clear_lums["midnight"],
                "{scene_name}/{rname}: noon-clear not brightest: {clear_lums:?}"
            );
            assert!(
                clear_lums["midnight"] < clear_lums["dawn"]
                    && clear_lums["midnight"] < clear_lums["dusk"]
                    && clear_lums["midnight"] < clear_lums["noon"],
                "{scene_name}/{rname}: midnight-clear not darkest: {clear_lums:?}"
            );
            // Overcast dims noon.
            assert!(
                noon_lums["thunderstorm"] < noon_lums["clear"],
                "{scene_name}/{rname}: noon thunderstorm not dimmer than clear: {noon_lums:?}"
            );
            let sheet_name = format!("sheet-{scene_name}-{rname}.png");
            sheet.save(out_dir.join(&sheet_name)).expect("save sheet");
            index.push_str(&format!(
                "\n{scene_name}/{rname}: clear luminance {clear_lums:?}; noon luminance {noon_lums:?}\n"
            ));
            println!("10.6 {scene_name}/{rname}: clear {clear_lums:?} noon {noon_lums:?}");
        }
    }
    std::fs::write(out_dir.join("index.txt"), index).expect("write index");
}

// ---------------------------------------------------------------------
// SkyScene seating from the wire
// ---------------------------------------------------------------------

#[test]
fn sky_scene_seats_from_wire_environment() {
    let wire = proto::samples::sample_environment();
    let scene = SkyScene::from_wire(&wire);
    assert_eq!(scene.game_env.turn, 309_600);
    assert_eq!(scene.game_env.weather_id, "weather_clear");
    assert_eq!(scene.game_env.time_of_day_sec, 43_200.0);
    // The seated scene evaluates: noon in spring, sun up.
    let state = scene.sky_state();
    assert!(state.sun_intensity > 0.0, "noon sun is up: {state:?}");
    // Snapshot piggyback seats the same scene.
    let snap = proto::samples::sample_snapshot();
    let scene2 = SkyScene::from_snapshot(&snap).expect("piggybacked environment");
    assert_eq!(scene2.game_env.turn, 309_600);
}
