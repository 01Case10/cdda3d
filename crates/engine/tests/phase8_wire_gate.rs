//! Phase 8 engine-wiring gates: the Phase 8 crates seated in
//! the live frame loop (the follow-up the Phase 8 record left to
//! the frame surface).
//!
//! - Vehicles: a `VehicleScene`'s models stamp into the world
//!   pass (raster meshes AND the RT tree) at their exported
//!   absolute part positions; the composed frame changes where
//!   the car is, and `FrameStats` reports the stamp.
//! - Transients: a `TransientScene`'s live emitters draw in the
//!   fields pass as fx shader volumes; an expired scene draws
//!   nothing and the frame is the baseline frame.
//! - Open air: `snapshot_world::build_columns` resolves floors
//!   through `hardcases::floor_at` — the roof of the terrain
//!   below, no fabricated floors in shafts — and the one built
//!   map feeds both renderers' world forms.
//!
//! GPU tests serialize on a lock and skip LOUDLY when no wgpu
//! adapter exists, mirroring frame_gate.rs.

use std::sync::Mutex;

use engine::capture::{CaptureArgs, RendererKind};
use engine::frame::{self, TransientScene, VehicleScene};
use engine::snapshot_world::{build_columns, ColumnSpec};

static GPU_LOCK: Mutex<()> = Mutex::new(());

const SIZE: (u32, u32) = (640, 360);

fn gpu_lock() -> std::sync::MutexGuard<'static, ()> {
    GPU_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn loud_skip(gate: &str, reason: &str) {
    let banner = format!(
        "================================================================\n\
         {gate}: NOT RUN — {reason}\n\
         This run proves NOTHING about the Phase 8 wiring. The gate\n\
         only counts on a machine with a wgpu adapter (lavapipe counts).\n\
         ================================================================"
    );
    eprintln!("{banner}");
    println!("{banner}");
}

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
                    label: Some("phase8-wire-gate-device"),
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

fn render(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    args: &CaptureArgs,
) -> (image::RgbaImage, frame::FrameOutcome) {
    let outcome = frame::render_frame(device, queue, args).expect("render_frame");
    let (img, _distinct) =
        frame::readback_rgba(device, queue, &outcome.texture, outcome.size).expect("readback");
    (img, outcome)
}

fn base_args() -> CaptureArgs {
    CaptureArgs {
        size: SIZE,
        camera_target: (6.0, 6.0),
        yaw_deg: 45.0,
        pitch_deg: 45.0,
        distance: 28.0,
        cutaway_z: 1,
        ..CaptureArgs::default()
    }
}

/// Fraction of pixels differing beyond the 2g.3 channel
/// tolerance (8) between two same-size frames.
fn mismatch_frac(a: &image::RgbaImage, b: &image::RgbaImage) -> f64 {
    assert_eq!(a.dimensions(), b.dimensions());
    let mut mismatched = 0usize;
    for (pa, pb) in a.pixels().zip(b.pixels()) {
        if (0..4).any(|c| (pa.0[c] as i32 - pb.0[c] as i32).unsigned_abs() > 8) {
            mismatched += 1;
        }
    }
    mismatched as f64 / (a.width() * a.height()).max(1) as f64
}

// ---------------------------------------------------------------------
// Open air (CPU): floor_at seated in world building
// ---------------------------------------------------------------------

fn def(id: u32, string_id: &str, roof: Option<&str>) -> proto::TileDefT {
    let mut d = proto::TileDefT::default();
    d.id = id;
    d.string_id = Some(string_id.into());
    d.roof_result = roof.map(|s| s.to_string());
    d
}

fn block_for(id: u32) -> mesher::Block {
    mesher::Block {
        id,
        color: [id as f32 / 10.0, 0.5, 0.5, 1.0],
        pass: mesher::Pass::Opaque,
        tex_layers: None,
    }
}

#[test]
fn openair_columns_resolve_floors_and_feed_both_renderers() {
    // Defs: grass (no roof), a wood floor whose roof is the
    // floor-above material, that material itself, open air.
    let mut defs = proto::TileDefsT::default();
    defs.defs = Some(vec![
        def(1, "t_grass", None),
        def(2, "t_wood_floor", Some("t_floor_above")),
        def(3, "t_floor_above", None),
        def(4, "t_open_air", None),
    ]);
    let table = hardcases::DefTable::from_tiledefs(&defs);
    assert_eq!(table.open_air_id, 4);

    let columns = vec![
        // Room: floor, then two open-air levels. z+1's floor is
        // the wood floor's roof; z+2 has none (below is air).
        ColumnSpec { x: 2, y: 2, z_min: 0, terrain: vec![2, 4, 4] },
        // Shaft: grass, then open air — grass declares no roof,
        // so no floor at either air level.
        ColumnSpec { x: 3, y: 2, z_min: 0, terrain: vec![1, 4, 4] },
        // Solid stack: the upper level's floor is its own tile.
        ColumnSpec { x: 4, y: 2, z_min: 0, terrain: vec![2, 2] },
    ];
    let scale = mesher::WorldScale::default();
    let map = build_columns(&table, &columns, scale, &block_for);
    let chunk = &map.chunks[&[0, 0]];
    let per = scale.blocks_per_z_level as i32;
    let id_at = |x: i32, y: i32, level: i32| {
        chunk
            .grid
            .get(x, y, level * per)
            .id
    };
    // Room column.
    assert_eq!(id_at(2, 2, 0), 2, "room floor is its own material");
    assert_eq!(id_at(2, 2, 1), 3, "open-air level's floor is the below terrain's roof");
    assert_eq!(id_at(2, 2, 2), 0, "open air over open air: no fabricated floor");
    // Shaft column.
    assert_eq!(id_at(3, 2, 0), 1);
    assert_eq!(id_at(3, 2, 1), 0, "roof-less terrain below: no floor");
    assert_eq!(id_at(3, 2, 2), 0, "the shaft stays open");
    // Solid column.
    assert_eq!(id_at(4, 2, 1), 2, "solid terrain is its own floor");

    // The same built map converts to the RT tree's world form
    // with the same floors — one builder, both renderers.
    let voxels = treebuild::VoxelWorld::from_chunk_map(&map);
    assert_eq!(
        voxels.get([2, 2, per]).map(|c| c.material_id),
        Some(3),
        "RT world carries the roof-material floor"
    );
    assert!(
        voxels.get([3, 2, per]).is_none(),
        "RT world has no phantom floor in the shaft"
    );
    // And it meshes (raster form) with real quads.
    let (meshes, stats) = engine::world::mesh_world(&map, 0, &[0, 1, 2]);
    assert!(meshes.total_quads() > 0 && stats.quads > 0);
    println!("openair wiring: floors resolve per the roof rule in both world forms");
}

// ---------------------------------------------------------------------
// Transients (CPU): emitter -> instance conversion
// ---------------------------------------------------------------------

#[test]
fn transient_instances_carry_params_and_positions() {
    let scene = TransientScene::fixture_burst(500);
    assert_eq!(scene.emitters.len(), 2, "both fixture emitters alive at 500 ms");
    let instances = engine::fxwire::transient_instances(&scene.emitters, scene.now_ms);
    assert_eq!(instances.len(), 2);
    // The explosion sits on its tile, in the flame class, with
    // the explosion's emissive strength.
    let boom = &instances[0];
    assert_eq!(boom.class_id, transient::CLASS_FLAME);
    assert_eq!(boom.center, [6.5, 1.8, -6.5]);
    assert!(boom.emissive_strength > 3.0);
    // The projectile is mid-flight: from (2,2) to (10,10) over
    // 1000 ms puts it on tile (6,6) at 500 ms — linear, because
    // the protocol carries endpoints + duration and no curve.
    let tracer = &instances[1];
    assert_eq!(tracer.class_id, transient::CLASS_ENERGY);
    assert_eq!(tracer.center, [6.5, 1.8, -6.5]);
    // After every duration, the scene is empty (no leftovers).
    let expired = TransientScene::fixture_burst(5_000);
    assert!(expired.emitters.is_empty());
    println!("transient wiring: params + interpolated positions reach the field instances");
}

// ---------------------------------------------------------------------
// 2d follow-up (CPU): open/broken state-subtile selection
// ---------------------------------------------------------------------

#[test]
fn state_subtile_selection_real_ultica_and_part_flags() {
    // Part flags -> subtile name (the game's ternary order:
    // open wins over broken — pinned cata_tiles.cpp).
    let car = VehicleScene::fixture_car([4, 5, 0], 0.0);
    let door = car.models[0]
        .parts
        .iter()
        .find(|p| p.key.vpart == vehicles::fixtures::VPART_DOOR)
        .expect("fixture car has a door part");
    assert_eq!(vehicles::part_state_subtile(door), None, "closed intact door: no state subtile");
    let mut open_door = *door;
    open_door.open = true;
    open_door.broken = true;
    assert_eq!(
        vehicles::part_state_subtile(&open_door),
        Some("open"),
        "open+broken selects open (game's order)"
    );
    let mut broken_door = *door;
    broken_door.broken = true;
    assert_eq!(vehicles::part_state_subtile(&broken_door), Some("broken"));

    // Real UltiCa data: vp_door's entry carries open (global
    // layer 23635) and broken (23634) subtile layers. The state
    // spec builder must put the OPEN layer's bytes in the open
    // variant's side slot — not the base side, not broken's.
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ultica_run1");
    if !dir.join("materials.json").exists() {
        println!("state-subtile real-data half NOT RUN — assets/ultica_run1 absent");
        return;
    }
    let table = engine::materials::MaterialTable::load(&dir).expect("ultica materials");
    let id_map: std::collections::HashMap<u32, String> =
        [(5000u32, "vp_door".to_string())].into_iter().collect();
    let assignments = [engine::textures::StateVariantAssignment {
        base_id: 5000,
        virtual_id: 5001,
        open: true,
        broken: false,
    }];
    let (spec, stats) =
        engine::textures::build_texture_spec_state_variants(&table, &dir, &id_map, &assignments)
            .expect("state spec");
    assert_eq!(stats.variants, 1);
    let base = spec.material_layers[5000];
    let variant = spec.material_layers[5001];
    assert_eq!(variant[3], 1, "variant entry present");
    assert_eq!(variant[0], base[0], "variant keeps base top");
    assert_eq!(variant[2], base[2], "variant keeps base bottom");
    assert_ne!(variant[1], base[1], "open variant side differs from base side");
    // The variant side's bytes are the open subtile layer's.
    let open_png = image::open(dir.join("blocks/layer_23635.png"))
        .expect("open layer png")
        .to_rgba8();
    assert_eq!(
        &spec.layers[variant[1] as usize][..64],
        &open_png.as_raw()[..64],
        "variant side bytes are vp_door's open subtile layer"
    );
    println!("2d follow-up: state selection proven on the fixture and on real vp_door data");
}

// ---------------------------------------------------------------------
// Per-block bake (GPU): the MEMORIZED mega-quad resolution limit
// ---------------------------------------------------------------------

fn project(
    cam: &engine::camera::OrbitCamera,
    p: glam::Vec3,
    size: (u32, u32),
) -> (f32, f32) {
    let vp = cam.view_projection(size.0 as f32 / size.1 as f32);
    let clip = vp * p.extend(1.0);
    let ndc = clip.truncate() / clip.w;
    (
        (ndc.x * 0.5 + 0.5) * size.0 as f32,
        (0.5 - ndc.y * 0.5) * size.1 as f32,
    )
}

fn patch_mean(img: &image::RgbaImage, cx: f32, cy: f32, r: i32) -> [f32; 3] {
    let (cx, cy) = (cx.round() as i32, cy.round() as i32);
    let (mut acc, mut n) = ([0f64; 3], 0u32);
    for y in (cy - r)..=(cy + r) {
        for x in (cx - r)..=(cx + r) {
            if x < 0 || y < 0 || x >= img.width() as i32 || y >= img.height() as i32 {
                continue;
            }
            let p = img.get_pixel(x as u32, y as u32).0;
            for c in 0..3 {
                acc[c] += p[c] as f64;
            }
            n += 1;
        }
    }
    assert!(n > 0, "patch at ({cx}, {cy}) is empty");
    [
        (acc[0] / n as f64) as f32,
        (acc[1] / n as f64) as f32,
        (acc[2] / n as f64) as f32,
    ]
}

fn saturation(m: [f32; 3]) -> f32 {
    let max = m[0].max(m[1]).max(m[2]);
    let min = m[0].min(m[1]).min(m[2]);
    if max <= 0.0 {
        0.0
    } else {
        (max - min) / max
    }
}

fn brightness(m: [f32; 3]) -> f32 {
    (m[0] + m[1] + m[2]) / 3.0
}

#[test]
fn gate_per_block_bake_renders_memorized_patch_at_tile_resolution() {
    let _guard = gpu_lock();
    let Some((device, queue)) = gpu() else {
        loud_skip("per-block-bake", "no wgpu adapter available");
        return;
    };
    // A 3x3 MEMORIZED patch in the middle of the wood floor —
    // inside the greedy-merged floor mega-quad, where the
    // per-vertex baker has no vertices (the recorded 2h.3
    // resolution limit).
    let memo: Vec<[i32; 3]> = (5..=7)
        .flat_map(|x| (5..=7).map(move |y| [x, y, 0]))
        .collect();
    let mut base = base_args();
    base.cutaway_z = 0;
    let cam = {
        let mut c = engine::camera::OrbitCamera::new(glam::Vec3::new(
            base.camera_target.0,
            1.0,
            -base.camera_target.1,
        ));
        c.yaw_deg = base.yaw_deg;
        c.pitch_deg = base.pitch_deg;
        c.distance = base.distance;
        c.clamp();
        c
    };
    let (px, py) = project(&cam, ui_share::picking::tile_center_world((6, 6, 0)), SIZE);

    let mut plain_args = base.clone();
    plain_args.lighting = Some(frame::LightingScene::fake_house(
        12.0,
        lighting::WeatherKind::Clear,
    ));
    let (plain_img, _) = render(&device, &queue, &plain_args);
    let plain_patch = patch_mean(&plain_img, px, py, 3);

    let marked = |mode: frame::BakeMode| {
        let mut lit = frame::LightingScene::fake_house_marked(
            12.0,
            lighting::WeatherKind::Clear,
            &[],
            &memo,
        );
        lit.bake_mode = mode;
        let mut a = base.clone();
        a.lighting = Some(lit);
        a
    };
    let (pv_img, _) = render(&device, &queue, &marked(frame::BakeMode::PerVertex));
    let pv_patch = patch_mean(&pv_img, px, py, 3);
    let (pb_img, _) = render(&device, &queue, &marked(frame::BakeMode::PerBlock));
    let pb_patch = patch_mean(&pb_img, px, py, 3);

    let (s_plain, s_pv, s_pb) = (
        saturation(plain_patch),
        saturation(pv_patch),
        saturation(pb_patch),
    );
    println!(
        "per-block bake: saturation plain {s_plain:.3} / per-vertex {s_pv:.3} / per-block {s_pb:.3}; \
         brightness plain {:.1} / per-block {:.1}",
        brightness(plain_patch),
        brightness(pb_patch)
    );
    // The recorded limit, asserted as the A/B difference: at the
    // mega-quad's center the per-vertex bake barely registers
    // the mark (its shading interpolates the quad's far corners).
    assert!(
        s_pv > s_plain * 0.7,
        "per-vertex center should stay near-plain (the recorded limit): {s_pv:.3} vs plain {s_plain:.3}"
    );
    // Per-block: the patch renders with the pinned MEMORIZED
    // presentation — saturation collapses to the 0.30 keep, and
    // the color is the pinned presentation of the floor albedo
    // itself (apply_visibility_rgb at the MEMORIZED factors,
    // sRGB-encoded), NOT a rescaling of the live shading: in
    // this scene the live floor is shaded to ~0.26 by the dark
    // level above it, so memory (0.45) is legitimately brighter
    // than the plain patch — the 2h.3 wall case runs the other
    // way. What is pinned is the presentation, not a direction.
    assert!(
        s_pb < s_plain * 0.55,
        "per-block MEMORIZED saturation must collapse: {s_pb:.3} vs plain {s_plain:.3}"
    );
    let albedo = mesher::block::block_for_tile(mesher::block::TILE_FLOOR_WOOD).color;
    let pinned = lighting::apply_visibility_rgb(
        [albedo[0], albedo[1], albedo[2]],
        lighting::Visibility::Memorized,
    );
    for c in 0..3 {
        let expected = pinned[c].powf(1.0 / 2.2) * 255.0;
        assert!(
            (pb_patch[c] - expected).abs() <= 12.0,
            "per-block MEMORIZED channel {c}: got {:.0}, pinned presentation {expected:.0}",
            pb_patch[c]
        );
    }
    println!("per-block bake: MEMORIZED patch renders at tile resolution");
}

// ---------------------------------------------------------------------
// GPU: vehicles + transients in the composed frame
// ---------------------------------------------------------------------

#[test]
fn gate_wire_vehicles_in_frame_both_renderers() {
    let _guard = gpu_lock();
    let Some((device, queue)) = gpu() else {
        loud_skip("wire-vehicles", "no wgpu adapter available");
        return;
    };
    for renderer in [RendererKind::Raster, RendererKind::Rt] {
        let mut plain = base_args();
        plain.renderer = renderer;
        // Close framing on the car's tiles: at the default
        // distance the whole car is ~100 px of frame, which is
        // signal but needlessly close to the rounding floor.
        plain.camera_target = (5.0, 5.0);
        plain.distance = 14.0;
        let (plain_img, _) = render(&device, &queue, &plain);

        let mut with_car = plain.clone();
        with_car.vehicles = Some(VehicleScene::fixture_car([4, 5, 0], 23.0));
        let (car_img, outcome) = render(&device, &queue, &with_car);
        assert_eq!(outcome.stats.vehicles, 1, "{renderer:?}: one vehicle stamped");
        assert!(
            outcome.stats.vehicle_tiles >= 8,
            "{renderer:?}: fixture car occupies its exported tiles (got {})",
            outcome.stats.vehicle_tiles
        );
        let frac = mismatch_frac(&plain_img, &car_img);
        println!("wire-vehicles {renderer:?}: frame delta {:.3}%", frac * 100.0);
        assert!(
            frac > 0.0005,
            "{renderer:?}: the stamped vehicle must be visible in the composed frame ({:.4}%)",
            frac * 100.0
        );
    }
    println!("wire-vehicles: fixture car renders in both renderers at its exported positions");
}

#[test]
fn gate_wire_transients_in_frame_and_expiry() {
    let _guard = gpu_lock();
    let Some((device, queue)) = gpu() else {
        loud_skip("wire-transients", "no wgpu adapter available");
        return;
    };
    let mut plain = base_args();
    plain.camera_target = (6.0, 6.0);
    plain.distance = 14.0;
    let (plain_img, _) = render(&device, &queue, &plain);

    // Live burst at 500 ms: the fields pass runs (no field defs
    // involved — transients alone seat the pass) and the frame
    // changes around the explosion.
    let mut live = plain.clone();
    live.transients = Some(TransientScene::fixture_burst(500));
    let (live_img, outcome) = render(&device, &queue, &live);
    assert_eq!(outcome.stats.transients, 2);
    assert!(
        outcome.stats.passes.contains(&frame::FramePass::Fields),
        "transients must seat the fields pass"
    );
    let frac = mismatch_frac(&plain_img, &live_img);
    println!("wire-transients: live frame delta {:.3}%", frac * 100.0);
    assert!(frac > 0.0005, "live emitters must be visible ({:.4}%)", frac * 100.0);

    // Expired: no instances, no fields pass, baseline frame.
    let mut dead = plain.clone();
    dead.transients = Some(TransientScene::fixture_burst(5_000));
    let (dead_img, outcome) = render(&device, &queue, &dead);
    assert_eq!(outcome.stats.transients, 0);
    assert!(
        !outcome.stats.passes.contains(&frame::FramePass::Fields),
        "an expired scene must not seat the fields pass"
    );
    let frac = mismatch_frac(&plain_img, &dead_img);
    assert!(
        frac <= 0.01,
        "expired emitters must leave the baseline frame (delta {:.4}%)",
        frac * 100.0
    );
    println!("wire-transients: burst renders live and expires clean");
}
