//! Phase 2h: the live frame loop — ONE offscreen composition
//! shared by the windowed app and the headless capture path.
//!
//! Every rendering layer landed as its own gated crate (world
//! raster/RT, fx field volumes + HD-2D post, billboards, the
//! lighting bake, UI compositing) but they had never shared a
//! frame: the windowed app drew the Phase 2 world alone, and the
//! capture path composed world → fields → post with no billboards,
//! no lighting, no UI. [`render_frame`] is the single composition
//! both paths now run, in the canonical order fixed by the Phase
//! 2g/4/6/7 records:
//!
//! **world → fields → billboards → post → UI**
//!
//! - *world*: raster meshes (`scene_prep`, optionally light-baked
//!   per Phase 7 §12) or the Sparse-Voxel-64Tree tracer
//!   (`rt::render_rt_frame`, optionally visibility-filtered).
//! - *fields*: the fx soft-particle field pass (2g.1), depth-tested
//!   against the world depth (for RT, the converted projected
//!   depth).
//! - *billboards*: `billboard::render::BillboardPass` (Phase 6),
//!   shadows then sprites, depth-tested against the same depth,
//!   projected with the *world* camera via
//!   `prepare_with_view_proj`.
//! - *post*: the fx HD-2D post stack (2g.1) — always the final
//!   *world* pass.
//! - *UI*: the resident `ui_share::UiPass` quad (Phase 4), drawn
//!   last, over the post-processed world, from the shared-memory
//!   store's current frame with its terrain rect masked per the
//!   frame itself.
//!
//! The windowed app renders through this function into an
//! offscreen texture and presents it with [`PresentBlit`]; the
//! headless capture reads the same texture back. That is §7's
//! capture-parity rule applied to the whole frame: a capture is
//! evidence about the windowed renderer because there is only one
//! composition. (This also closes 2g.1 deviation 1: the windowed
//! path now has the offscreen + post route.)
//!
//! Live game data seats into this loop through `CaptureArgs`'
//! `lighting` / `billboards` / `ui` inputs when the fork runs
//! (0.1/0.2); the gates run it on stub/snapshot data (fake house,
//! fixture + real-UltiCa crowd scenes, the Phase 7 fake-house
//! light grid, a synthetic UI store frame).

use std::time::{Duration, Instant};

use glam::Vec3;

use crate::camera::OrbitCamera;
use crate::capture::{CaptureArgs, CaptureError, RendererKind};
use crate::cutaway::CutawayMode;
use crate::overlay::MeshStats;
use crate::render::{self, Renderer, CAPTURE_FORMAT};

// ---------------------------------------------------------------------
// Pass order (gate 2h.1: asserted in code structure)
// ---------------------------------------------------------------------

/// One layer of the composed frame, in draw order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FramePass {
    World,
    Fields,
    Billboards,
    Post,
    Ui,
}

/// The canonical frame order. [`render_frame`] executes its steps
/// in exactly this sequence and records the executed steps in
/// [`FrameStats::passes`]; the gate asserts the two agree.
pub const FRAME_ORDER: [FramePass; 5] = [
    FramePass::World,
    FramePass::Fields,
    FramePass::Billboards,
    FramePass::Post,
    FramePass::Ui,
];

// ---------------------------------------------------------------------
// Scene inputs (stub/snapshot data until the fork runs)
// ---------------------------------------------------------------------

/// Lighting input for one frame (Phase 2h.3): the per-tile game
/// light grid (`lit_level` + seen — the game's visibility truth),
/// the environment state (day/night hour × weather) the bake is
/// evaluated under, and the game z of chunk level 0 (the grid is
/// indexed in game tile coordinates).
/// Which lighting baker the raster world pass uses (Phase 7
/// shipped both; the frame loop originally wired only one).
///
/// - `PerVertex` (default, the 2h wiring): `bake_pass_meshes`
///   after greedy meshing — air-side sampling, leak-proof across
///   a wall, but its resolution is the *mesh vertex density*: a
///   light change inside one greedy-merged mega-quad (a
///   MEMORIZED patch in the middle of a floor) has no vertices
///   to land on and changes 0 pixels (the recorded 2h.3 limit).
/// - `PerBlock`: `bake_chunk_map_with_offset` before meshing —
///   each block carries its own tile's presented light, so the
///   greedy mesher cannot merge across a light boundary and
///   every tile renders at tile resolution. The cost is the
///   per-block baker's known trade: a wall block takes its own
///   tile's centre sample on both faces (no air-side split).
///
/// The RT path is per-voxel in both modes and ignores this.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BakeMode {
    #[default]
    PerVertex,
    PerBlock,
}

#[derive(Debug, Clone)]
pub struct LightingScene {
    pub grid: lighting::LightGrid,
    pub env: lighting::Environment,
    pub min_game_z: i32,
    pub bake_mode: BakeMode,
}

impl LightingScene {
    /// The fake house under the Phase 7 night-interior light grid
    /// (exterior `DARK`, lamp falloff inside, walls occluding),
    /// evaluated at `hour` in `weather`.
    pub fn fake_house(hour: f32, weather: lighting::WeatherKind) -> Self {
        Self {
            grid: lighting::fixtures::fake_house_night_light(),
            env: lighting::Environment::new(hour, weather),
            min_game_z: crate::world::HOUSE_MIN_GAME_Z,
            bake_mode: BakeMode::PerVertex,
        }
    }

    /// [`LightingScene::fake_house`] with specific tiles forced
    /// `BLANK` (never seen) and/or `MEMORIZED` (remembered) — the
    /// gate 2h.3 scene. Memorized tiles keep their occlusion flag.
    pub fn fake_house_marked(
        hour: f32,
        weather: lighting::WeatherKind,
        blank: &[[i32; 3]],
        memorized: &[[i32; 3]],
    ) -> Self {
        let mut s = Self::fake_house(hour, weather);
        for t in blank {
            s.grid.set(t[0], t[1], t[2], lighting::TileLight::BLANK);
        }
        for t in memorized {
            let occ = s.grid.get(t[0], t[1], t[2]).occluding;
            s.grid.set(
                t[0],
                t[1],
                t[2],
                lighting::TileLight::new(proto::LitLevel::MEMORIZED, 128).occluding(occ),
            );
        }
        s
    }
}

/// Billboard input for one frame (Phase 2h.2): a sprite atlas
/// (fixture or the voxelizer's real UltiCa output), the entities
/// to draw (protocol `Entity` shape, via billboard's mirror type),
/// optional per-tile item piles, and the animation time.
#[derive(Debug, Clone)]
pub struct BillboardScene {
    pub atlas: billboard::atlas::SpriteAtlas,
    pub entities: Vec<billboard::instance::FakeEntity>,
    pub items: Option<billboard::instance::FakeItemTiles>,
    pub time_s: f32,
}

impl BillboardScene {
    /// Fixture-atlas crowd inside the fake house (hermetic — the
    /// composed-golden scene): an equipped zombie, a skeleton, a
    /// ghost, an oversized hulk, plus the fixture item piles.
    pub fn fixture_crowd() -> Self {
        use billboard::composite::Facing;
        use billboard::instance::FakeEntity;
        let mut z1 = FakeEntity::creature(1, "mon_zombie", [6, 5, 0]);
        z1.overlays = vec!["overlay_jacket".into(), "overlay_jeans".into()];
        let mut z3 = FakeEntity::creature(3, "mon_zombie", [5, 7, 0]);
        z3.facing = Facing::Left;
        Self {
            atlas: billboard::atlas::fixture_atlas(),
            entities: vec![
                z1,
                FakeEntity::creature(2, "mon_skeleton", [8, 6, 0]),
                z3,
                FakeEntity::creature(4, "mon_ghost", [4, 7, 0]),
                FakeEntity::creature(5, "mon_hulk", [9, 4, 0]),
            ],
            items: Some(billboard::scene::crowd_items()),
            time_s: 0.0,
        }
    }

    /// Real-UltiCa crowd for gate 2h.2, in the fake house's
    /// geometry (the Phase 6 real-capture composition seated in
    /// the engine's world): an equipped zombie and the hulk
    /// *inside*; one skeleton *outside* the north wall, fully
    /// occluded by it; one zombie *outside* the north wall behind
    /// the window tile at game (3, 0), visible through the
    /// window's transparent pass. Sprite ids are the real-capture
    /// set, verified present in the atlas by the caller/gate.
    pub fn ultica_crowd(dir: &std::path::Path) -> Result<Self, String> {
        use billboard::instance::FakeEntity;
        let atlas = billboard::atlas::SpriteAtlas::load(dir)?;
        let mut z1 = FakeEntity::creature(1, "mon_zombie_1", [6, 5, 0]);
        z1.overlays = vec!["overlay_effect_bleed".into()];
        Ok(Self {
            atlas,
            entities: vec![
                z1,
                // Behind the north wall (opaque): must not appear.
                FakeEntity::creature(7, "mon_skeleton", [1, -1, 0]),
                // Behind the north window (transparent): visible.
                FakeEntity::creature(6, "mon_zombie_2", [3, -1, 0]),
                // The picking target (oversized 64x96 sprite).
                FakeEntity::creature(8, "mon_zombie_hulk", [9, 4, 0]),
                FakeEntity::creature(10, "mon_dog", [10, 6, 0]),
            ],
            items: None,
            time_s: 0.0,
        })
    }
}

/// Vehicle input for one frame (Phase 8 wiring): the vehicle
/// models to stamp into the world pass, in their exported state
/// (absolute part positions verbatim — the engine never
/// re-derives them from facing + mounts, the Phase 8 rule).
#[derive(Debug, Clone)]
pub struct VehicleScene {
    pub models: Vec<vehicles::VehicleModel>,
}

impl VehicleScene {
    /// Every vehicle currently in a `VehicleStore` (the live
    /// path once the fork seats data in the loop).
    pub fn from_store(store: &vehicles::VehicleStore) -> Self {
        Self {
            models: store.ids().filter_map(|id| store.get(id).cloned()).collect(),
        }
    }

    /// The Phase 8 fixture car at `pivot`, facing `face_deg`
    /// (mock game truth from the vehicles crate — the same
    /// fixture its gates prove the model against).
    pub fn fixture_car(pivot: [i32; 3], face_deg: f32) -> Self {
        let wire = vehicles::fixtures::truth_vehicle(1, pivot, face_deg);
        Self {
            models: vec![vehicles::VehicleModel::from_proto(&wire)],
        }
    }
}

/// Transient-emitter input for one frame (Phase 8 wiring,
/// gate 8.5's presentation half): the emitters alive at
/// `now_ms` in a `transient::TransientSystem` fed from
/// `Effects` messages. The scene owns a snapshot of the active
/// set, so a frame is a pure function of its inputs.
#[derive(Debug, Clone)]
pub struct TransientScene {
    pub emitters: Vec<transient::Emitter>,
    pub now_ms: u64,
}

impl TransientScene {
    /// Feed `Effects` messages into a fresh system in order,
    /// advance its clock to `now_ms`, and snapshot the active
    /// emitters (advance-before-ingest ordering per the soak
    /// driver's recorded lesson: ingest stamps emitters with
    /// the system's current time, so each message is ingested
    /// at the clock value the caller pairs with it — here all
    /// messages are ingested at t=0 and the clock then advances
    /// to `now_ms`, expiring what has finished).
    pub fn from_effects(messages: &[proto::EffectsT], now_ms: u64) -> Self {
        let mut system = transient::TransientSystem::new();
        for m in messages {
            system.ingest(m);
        }
        system.advance(now_ms);
        Self {
            emitters: system.active().to_vec(),
            now_ms,
        }
    }

    /// A deterministic demo burst for captures and the gate: an
    /// explosion on the fake-house floor tile (6, 6, 0) and a
    /// tracer projectile crossing the house, both spawned at
    /// t=0; at `now_ms` the projectile is mid-flight.
    pub fn fixture_burst(now_ms: u64) -> Self {
        let mut boom = proto::EffectEventT::default();
        boom.kind = proto::EffectKind::Explosion;
        boom.effect_id = Some("explosion".into());
        boom.from_pos = Some(proto::Vec3iT { x: 6, y: 6, z: 0 });
        boom.to_pos = Some(proto::Vec3iT { x: 6, y: 6, z: 0 });
        boom.duration_ms = 2000;
        let mut tracer = proto::EffectEventT::default();
        tracer.kind = proto::EffectKind::Projectile;
        tracer.effect_id = Some("bullet_tracer".into());
        tracer.from_pos = Some(proto::Vec3iT { x: 2, y: 2, z: 0 });
        tracer.to_pos = Some(proto::Vec3iT { x: 10, y: 10, z: 0 });
        tracer.duration_ms = 1000;
        let mut fx = proto::EffectsT::default();
        fx.seq = 1;
        fx.events = Some(vec![boom, tracer]);
        Self::from_effects(&[fx], now_ms)
    }
}

/// One UI frame prepared for compositing (Phase 2h.4): the store
/// snapshot's tight pixels with the terrain rect already masked
/// (alpha zeroed inside the rect *the frame itself* reported),
/// plus the validated notification data the input router needs.
#[derive(Debug, Clone)]
pub struct UiPrepared {
    pub tight: Vec<u8>,
    pub size: (u32, u32),
    pub mode: proto::UiMode,
    pub terrain_rect: Option<ui_share::TerrainRect>,
    pub seq: u64,
}

/// Build the stub UI store the gates and the windowed demo run
/// against: an in-memory `UiShmStore` holding one synthetic
/// gameplay frame (sidebar + message log, terrain rect unpainted
/// — Route B's shape), plus its `UiFrame` notification. With
/// `opaque_terrain`, the terrain rect is painted opaque red in
/// the stored frame first: compositing must still show the world
/// there, because the mask comes from the frame's own reported
/// rect, not from the stored alpha (gate 2h.4).
pub fn stub_ui_store(
    w: u32,
    h: u32,
    opaque_terrain: bool,
) -> Result<(ui_share::UiShmStore, proto::UiFrameT), String> {
    let store = ui_share::UiShmStore::create_memory(w, h).map_err(|e| e.to_string())?;
    let mut canvas = ui_share::synthetic::draw_gameplay_sidebar(w, h);
    let terrain = ui_share::synthetic::synthetic_terrain_rect(w, h);
    if opaque_terrain {
        canvas.fill_rect(terrain.x, terrain.y, terrain.w, terrain.h, [200, 30, 30, 255]);
    }
    store
        .write_frame_tight(
            &canvas.pixels,
            w,
            h,
            ui_share::DirtyRect::full(w, h),
        )
        .map_err(|e| e.to_string())?;
    let snap = store.read_latest().map_err(|e| e.to_string())?;
    let note = ui_share::frame::frame_notification(&snap, proto::UiMode::MapInput, Some(terrain));
    Ok((store, note))
}

/// The engine's per-notification UI path (docs/phase4-ui.md wiring
/// step 3): validate the `UiFrame` notification against the store,
/// read the newest complete frame, mask its terrain rect. The
/// result is what [`render_frame`] composites and what the input
/// router is configured from.
pub fn prepare_ui(
    store: &ui_share::UiShmStore,
    note: &proto::UiFrameT,
) -> Result<UiPrepared, String> {
    let validated = ui_share::frame::validate_frame(store, note).map_err(|e| e.to_string())?;
    let snap = store.read_latest().map_err(|e| e.to_string())?;
    let tight = ui_share::frame::masked_tight_pixels(&snap, validated.terrain_rect);
    Ok(UiPrepared {
        tight,
        size: (snap.width, snap.height),
        mode: validated.mode,
        terrain_rect: validated.terrain_rect,
        seq: validated.seq,
    })
}

// ---------------------------------------------------------------------
// Picking (engine side of gates 2h.2 / 2h.4)
// ---------------------------------------------------------------------

/// The `ui-share` picking camera for the engine's orbit camera
/// (same target/yaw/pitch/distance; the raster/RT FOV is 60°).
pub fn pick_camera(camera: &OrbitCamera) -> ui_share::picking::PickCamera {
    ui_share::picking::PickCamera {
        target: camera.target,
        yaw_deg: camera.yaw_deg,
        pitch_deg: camera.pitch_deg,
        distance: camera.distance,
        fov_y_deg: 60.0,
    }
}

/// Occupancy predicate over the fake-house chunk map, in tile
/// coordinates: a tile is occupied when any block of its z-level
/// segment is non-air (the DDA's stop condition).
pub fn fake_house_occupied() -> impl Fn(i32, i32, i32) -> bool {
    let map = crate::world::build_fake_house(mesher::scale::WorldScale::default());
    move |x: i32, y: i32, z: i32| {
        let per = mesher::scale::WorldScale::default().blocks_per_z_level as i32;
        let level = z - crate::world::HOUSE_MIN_GAME_Z;
        if level < 0 {
            return false;
        }
        let coord = [
            x.div_euclid(mesher::chunk::CHUNK_W as i32),
            y.div_euclid(mesher::chunk::CHUNK_H as i32),
        ];
        let Some(chunk) = map.chunks.get(&coord) else {
            return false;
        };
        let (lx, ly) = (
            x.rem_euclid(mesher::chunk::CHUNK_W as i32),
            y.rem_euclid(mesher::chunk::CHUNK_H as i32),
        );
        (0..per).any(|b| {
            !chunk
                .grid
                .get(lx, ly, level * per + b)
                .is_air()
        })
    }
}

/// Alpha picking against a billboard scene (gate 2h.2's engine
/// half): compose each entity through a fresh `CompositeCache`
/// (the same cache the render path feeds from exported overlay
/// lists), build its instance, and return the nearest entity whose
/// *opaque* sprite pixels the ray crosses — transparent pixels
/// pass through.
pub fn pick_billboard_entity(
    scene: &BillboardScene,
    camera: &OrbitCamera,
    ray_origin: Vec3,
    ray_dir: Vec3,
) -> Option<u64> {
    let eye = camera.eye();
    let mut cache = billboard::composite::CompositeCache::new();
    let mut owned = Vec::new();
    for e in &scene.entities {
        let (comp, _missing) =
            cache.get_or_compose(&scene.atlas, &e.sprite_id, &e.overlays, e.facing);
        let inst = billboard::instance::entity_instance(
            e,
            &comp,
            [0.0, 0.0, 1.0, 1.0],
            eye,
            scene.time_s,
            billboard::instance::TILE_PIXEL_SIZE,
        );
        owned.push((inst, comp));
    }
    let refs: Vec<(
        &billboard::instance::BillboardInstance,
        &billboard::composite::Composite,
    )> = owned.iter().map(|(i, c)| (i, c.as_ref())).collect();
    billboard::instance::pick_any(ray_origin, ray_dir, &refs, eye)
}

// ---------------------------------------------------------------------
// Frame outcome
// ---------------------------------------------------------------------

/// What one composed frame measured about itself.
#[derive(Debug, Clone, Default)]
pub struct FrameStats {
    /// Passes actually executed, in execution order — a subsequence
    /// of [`FRAME_ORDER`] (equal to it when every layer is on).
    pub passes: Vec<FramePass>,
    /// Billboard instances drawn (creatures + item piles).
    pub billboards: usize,
    /// Time inside the UI stage (upload + composite + GPU wait) —
    /// gate 4.12's in-loop measurement point (Phase 2h.4).
    pub ui_stage: Duration,
    /// Raster mesh stats for the world pass (RT reports leaves in
    /// `FrameOutcome::quads` instead).
    pub mesh_stats: Option<MeshStats>,
    /// Phase 8 wiring: vehicle models stamped into the world
    /// pass, and the tiles they occupy.
    pub vehicles: usize,
    pub vehicle_tiles: usize,
    /// Phase 8 wiring: transient emitters drawn in the fields
    /// pass.
    pub transients: usize,
}

/// One composed frame: the final color texture (post-UI), ready
/// to present ([`PresentBlit`]) or read back (capture).
pub struct FrameOutcome {
    pub texture: wgpu::Texture,
    pub size: (u32, u32),
    /// Raster: mesh quads drawn. RT: tree leaves (voxels) traced.
    pub quads: usize,
    pub stats: FrameStats,
}

// ---------------------------------------------------------------------
// Shared helpers (moved from capture.rs — one composition, one copy)
// ---------------------------------------------------------------------

/// The effect defs a frame runs with, if fx is enabled: direct
/// defs win over the `effects.json` path.
pub(crate) fn fx_defs(
    args: &CaptureArgs,
) -> Result<Option<Vec<fx::effect::EffectDef>>, CaptureError> {
    if let Some(defs) = &args.effects_defs {
        return Ok(Some(defs.clone()));
    }
    match &args.effects {
        Some(path) => Ok(Some(
            crate::fxwire::load_defs(path).map_err(CaptureError::Device)?,
        )),
        None => Ok(None),
    }
}

/// The frame's orbit camera (shared framing for both renderers —
/// the RT camera is built from the same values).
pub(crate) fn capture_camera(args: &CaptureArgs) -> OrbitCamera {
    let mut camera = OrbitCamera::new(Vec3::new(args.camera_target.0, 1.0, -args.camera_target.1));
    camera.yaw_deg = args.yaw_deg;
    camera.pitch_deg = args.pitch_deg;
    camera.distance = args.distance;
    camera.clamp();
    camera
}

/// Read an offscreen texture back into an image (+ its distinct
/// color count).
pub fn readback_rgba(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    size: (u32, u32),
) -> Result<(image::RgbaImage, usize), CaptureError> {
    let (width, height) = size;
    let unpadded = width * 4;
    let padded = unpadded.div_ceil(256) * 256;
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("frame-staging"),
        size: (padded * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("frame-copy"),
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);
    device.poll(wgpu::Maintain::Wait);

    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    device.poll(wgpu::Maintain::Wait);
    rx.recv()
        .map_err(|_| CaptureError::Readback("map channel closed".into()))?
        .map_err(|e| CaptureError::Readback(e.to_string()))?;

    let mut pixels = Vec::with_capacity((unpadded * height) as usize);
    {
        let data = slice.get_mapped_range();
        for row in 0..height as usize {
            let start = row * padded as usize;
            pixels.extend_from_slice(&data[start..start + unpadded as usize]);
        }
    }
    staging.unmap();

    let distinct: std::collections::HashSet<[u8; 4]> = pixels
        .chunks_exact(4)
        .map(|c| [c[0], c[1], c[2], c[3]])
        .collect();
    let img = image::RgbaImage::from_raw(width, height, pixels)
        .ok_or_else(|| CaptureError::Readback("pixel buffer size mismatch".into()))?;
    Ok((img, distinct.len()))
}

// ---------------------------------------------------------------------
// The composition
// ---------------------------------------------------------------------

/// Render one composed frame on an existing device/queue. This is
/// the only frame composition in the engine: the capture path and
/// the windowed app both call it (see the module docs).
pub fn render_frame(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    args: &CaptureArgs,
) -> Result<FrameOutcome, CaptureError> {
    let defs = fx_defs(args)?;
    let mut stats = FrameStats::default();

    if args.renderer == RendererKind::Rt {
        // World (RT): trace the frame, copy its color into an
        // engine-format target, and — when a downstream pass needs
        // depth (fields depth-fade, billboard depth test, post
        // DoF) — convert the tracer's linear depth buffer into the
        // projected-depth texture those passes consume (2g.1).
        let frame = crate::rt::render_rt_frame(device, queue, args)?;
        let (width, height) = frame.size;
        let color_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("frame-rt-color"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: CAPTURE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("frame-rt-copy"),
        });
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &frame.color,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &color_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        queue.submit([encoder.finish()]);
        stats.passes.push(FramePass::World);

        let needs_depth =
            defs.is_some() || args.billboards.is_some() || args.transients.is_some();
        let depth = if needs_depth {
            let (depth_texture, depth_view) =
                render::create_depth_texture(device, (width, height));
            crate::fxwire::RtDepthConvert::new(device).draw(
                device,
                queue,
                &frame.depth,
                frame.max_dist,
                &frame.camera,
                (width, height),
                &depth_view,
            );
            Some((depth_texture, depth_view))
        } else {
            None
        };
        if let Some(v) = &args.vehicles {
            stats.vehicles = v.models.len();
            let tiles: std::collections::BTreeSet<[i32; 3]> = v
                .models
                .iter()
                .flat_map(|m| m.parts.iter().map(|p| p.abs_pos))
                .collect();
            stats.vehicle_tiles = tiles.len();
        }
        let camera = capture_camera(args);
        let texture = composite_overlays(
            device,
            queue,
            args,
            defs.as_deref(),
            &camera,
            color_texture,
            depth,
            (width, height),
            &mut stats,
        )?;
        return Ok(FrameOutcome {
            texture,
            size: (width, height),
            quads: frame.leaves,
            stats,
        });
    }

    // World (raster): same world, same meshing as the pre-2h
    // capture — scene preparation is shared with the windowed
    // remesh (`scene_prep`), and with a lighting scene it is the
    // lit preparation (visibility mask + per-vertex bake, 2h.3).
    let visible = CutawayMode::HideAbove(0).visible_levels(args.cutaway_z);
    let mut renderer = Renderer::new(device, CAPTURE_FORMAT);
    let (meshes, mstats, spec) = match &args.lighting {
        Some(lit) => crate::scene_prep::prepare_raster_house_lit_v(
            &visible,
            args.textures.as_deref(),
            lit,
            args.vehicles.as_ref(),
        )
        .map_err(CaptureError::Device)?,
        None => crate::scene_prep::prepare_raster_house_v(
            &visible,
            args.textures.as_deref(),
            args.vehicles.as_ref(),
        )
        .map_err(CaptureError::Device)?,
    };
    if let Some(v) = &args.vehicles {
        stats.vehicles = v.models.len();
        let tiles: std::collections::BTreeSet<[i32; 3]> = v
            .models
            .iter()
            .flat_map(|m| m.parts.iter().map(|p| p.abs_pos))
            .collect();
        stats.vehicle_tiles = tiles.len();
    }
    if let Some(spec) = &spec {
        renderer.set_textures(device, queue, spec);
    }
    let gpu_meshes = renderer.upload(device, &meshes);
    let quads = mstats.quads;
    stats.mesh_stats = Some(mstats);

    let (width, height) = (args.size.0.max(1), args.size.1.max(1));
    let color_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("frame-color"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: CAPTURE_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let color_view = color_texture.create_view(&Default::default());
    let (depth_texture, depth_view) = render::create_depth_texture(device, (width, height));

    let camera = capture_camera(args);
    let aspect = width as f32 / height as f32;
    renderer.set_view_projection(queue, camera.view_projection(aspect));
    renderer.draw(device, queue, &gpu_meshes, &color_view, &depth_view);
    stats.passes.push(FramePass::World);

    let texture = composite_overlays(
        device,
        queue,
        args,
        defs.as_deref(),
        &camera,
        color_texture,
        Some((depth_texture, depth_view)),
        (width, height),
        &mut stats,
    )?;
    Ok(FrameOutcome {
        texture,
        size: (width, height),
        quads,
        stats,
    })
}

/// Steps 2–5 of the frame, shared by both renderers: fields,
/// billboards, post, UI — in [`FRAME_ORDER`]. `color` is the world
/// pass's output; `depth` is the world depth (projected) the
/// overlay passes test against, present whenever any step that
/// needs it can run. Returns the final texture (the post output
/// when post ran, otherwise the world color texture with the
/// overlays drawn into it).
#[allow(clippy::too_many_arguments)]
fn composite_overlays(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    args: &CaptureArgs,
    defs: Option<&[fx::effect::EffectDef]>,
    camera: &OrbitCamera,
    color: wgpu::Texture,
    depth: Option<(wgpu::Texture, wgpu::TextureView)>,
    size: (u32, u32),
    stats: &mut FrameStats,
) -> Result<wgpu::Texture, CaptureError> {
    let (width, height) = size;
    let aspect = width as f32 / height as f32;
    let mut current = color;

    // -- Fields (soft-particle volumes over the world) -----------
    // Field volumes (2g.1) and transient emitters (Phase 8 wiring)
    // share this pass: both are soft volumes in the fx shader
    // classes, depth-tested against the world. The pass runs when
    // either source produces instances; with neither, the frame
    // is exactly the pre-wiring one.
    {
        let mut instances: Vec<fx::render::FieldInstGpu> = Vec::new();
        let mut fields_ran = false;
        if let Some(defs) = defs {
            if args.fx_fields {
                let fields =
                    crate::fxwire::fake_house_fields(defs).map_err(CaptureError::Device)?;
                instances.extend(crate::fxwire::sorted_instances(&fields, camera.eye()));
                fields_ran = true;
            }
        }
        if let Some(ts) = &args.transients {
            let before = instances.len();
            instances.extend(crate::fxwire::transient_instances(&ts.emitters, ts.now_ms));
            stats.transients = instances.len() - before;
        }
        if let Some((depth_texture, depth_view)) = &depth {
            if !instances.is_empty() {
                // One combined back-to-front sort: with fields
                // only, this reproduces the pre-wiring order
                // exactly (the sort is stable and the field
                // instances arrive pre-sorted by the same key).
                crate::fxwire::sort_instances(&mut instances, camera.eye());
                // The soft-particle pass samples a COPY of the
                // scene depth (a pass cannot sample its own depth
                // attachment).
                let depth_copy = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("frame-depth-copy"),
                    size: wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: render::DEPTH_FORMAT,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                let depth_copy_view = depth_copy.create_view(&Default::default());
                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("frame-depth-copy-encoder"),
                    });
                encoder.copy_texture_to_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: depth_texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::TexelCopyTextureInfo {
                        texture: &depth_copy,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                );
                queue.submit([encoder.finish()]);

                let color_view = current.create_view(&Default::default());
                let pass = fx::passes::FieldPass::new(device, CAPTURE_FORMAT, render::DEPTH_FORMAT);
                let uniforms = fx::passes::FieldUniforms::new(
                    camera.view_projection(aspect),
                    camera.eye(),
                    args.fx_time,
                    fx::passes::soft_fade_z(camera.distance, 0.35),
                );
                let prepared = pass.prepare(device, &instances, &uniforms, &depth_copy_view);
                pass.draw(device, queue, &prepared, &color_view, depth_view);
            }
        }
        if fields_ran || stats.transients > 0 {
            stats.passes.push(FramePass::Fields);
        }
    }

    // -- Billboards (shadows, then sprites; world-depth tested) --
    if let Some(scene) = &args.billboards {
        let (_, depth_view) = depth.as_ref().expect(
            "billboards require world depth (raster depth texture or converted RT depth)",
        );
        let color_view = current.create_view(&Default::default());
        stats.billboards = draw_billboards(
            device,
            queue,
            scene,
            camera,
            &color_view,
            depth_view,
            size,
        )?;
        stats.passes.push(FramePass::Billboards);
    }

    // -- Post (HD-2D stack; the final world pass) ----------------
    if let Some(_defs) = defs {
        if args.fx_post {
            let (_, depth_view) = depth.as_ref().expect("post requires world depth");
            let out_tex = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("frame-post-out"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: CAPTURE_FORMAT,
                // TEXTURE_BINDING: the post output can be the
                // final frame texture, which PresentBlit samples.
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let out_view = out_tex.create_view(&Default::default());
            let color_view = current.create_view(&Default::default());
            let post = fx::passes::PostPass::new(device, CAPTURE_FORMAT);
            post.draw(
                device,
                queue,
                &color_view,
                depth_view,
                &out_view,
                &fx::post::PostConfig::default(),
                fx::passes::projected_depth(camera.distance),
                size,
            );
            current = out_tex;
            stats.passes.push(FramePass::Post);
        }
    }

    // -- UI (last pass, over the post-processed world) -----------
    if let Some(ui) = &args.ui {
        // Drain the world passes' queued work first: gate 4.12
        // budgets the UI *stage* (upload + composite), and timing
        // a poll over an undrained queue would charge the whole
        // frame's GPU time to the UI — on a software adapter that
        // is ~100 ms of world/fx work wearing the UI's name. With
        // the queue drained, the stage time below is the stage's
        // own cost, the in-loop equivalent of Phase 4's harness
        // segment.
        device.poll(wgpu::Maintain::Wait);
        let t0 = Instant::now();
        let mut pass = ui_share::UiPass::new(device, CAPTURE_FORMAT);
        pass.upload(device, queue, &ui.tight, ui.size, None);
        let view = current.create_view(&Default::default());
        pass.draw(
            device,
            queue,
            &view,
            size,
            ui_share::ScalingMode::StretchNearest,
        );
        // The in-loop cost includes the GPU finishing the
        // composite (what a presented frame would wait on).
        device.poll(wgpu::Maintain::Wait);
        stats.ui_stage = t0.elapsed();
        stats.passes.push(FramePass::Ui);
    }

    Ok(current)
}

/// Draw one billboard scene into the composed frame: creatures
/// from their packed overlay composites first, then item piles
/// from the raw atlas — two prepares/draws of the one pass, each
/// in its own Load/Load render pass over the world color+depth.
fn draw_billboards(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    scene: &BillboardScene,
    camera: &OrbitCamera,
    color_view: &wgpu::TextureView,
    depth_view: &wgpu::TextureView,
    size: (u32, u32),
) -> Result<usize, CaptureError> {
    // Instance generation (sorting, bob, shadows) reads the eye
    // through billboard's camera type; the projection the pass
    // draws with is the engine's (prepare_with_view_proj).
    let bb_cam = billboard::render::Camera {
        target: camera.target,
        yaw_deg: camera.yaw_deg,
        pitch_deg: camera.pitch_deg,
        distance: camera.distance,
    };
    let built = billboard::scene::build_frame(&scene.atlas, &scene.entities, &bb_cam, scene.time_s);
    let aspect = size.0 as f32 / size.1.max(1) as f32;
    let view_proj = camera.view_projection(aspect);
    let eye = camera.eye();
    let mut pass = billboard::render::BillboardPass::new(device, CAPTURE_FORMAT, render::DEPTH_FORMAT);
    let mut count = 0usize;

    pass.prepare_with_view_proj(
        device,
        queue,
        view_proj,
        eye,
        scene.time_s,
        &built.packed,
        &built.instances,
        &built.shadows,
    );
    draw_billboard_pass(device, queue, &pass, color_view, depth_view);
    count += built.instances.len();

    if let Some(items) = &scene.items {
        let inst = billboard::instance::item_instances(items, &scene.atlas, eye);
        if !inst.is_empty() {
            pass.prepare_with_view_proj(
                device,
                queue,
                view_proj,
                eye,
                scene.time_s,
                &scene.atlas.image,
                &inst,
                &[],
            );
            draw_billboard_pass(device, queue, &pass, color_view, depth_view);
            count += inst.len();
        }
    }
    Ok(count)
}

/// One billboard draw inside its own render pass (color + depth
/// loaded from the world pass, never cleared).
fn draw_billboard_pass(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pass: &billboard::render::BillboardPass,
    color_view: &wgpu::TextureView,
    depth_view: &wgpu::TextureView,
) {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("frame-billboard-encoder"),
    });
    {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("frame-billboard-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.draw(&mut rpass, billboard::render::AlphaMode::Blend);
    }
    queue.submit([encoder.finish()]);
}

// ---------------------------------------------------------------------
// Presentation (windowed path)
// ---------------------------------------------------------------------

const BLIT_WGSL: &str = r#"
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var out: VsOut;
    let x = f32(i32(vi & 1u) * 4 - 1);
    let y = f32(i32(vi & 2u) * 2 - 1);
    out.clip = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x * 0.5 + 0.5, 0.5 - y * 0.5);
    return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    return textureSample(tex, samp, input.uv);
}
"#;

/// Fullscreen-triangle present of a composed frame texture onto a
/// surface. The composed texture is `Rgba8UnormSrgb`: sampling it
/// decodes sRGB → linear and the sRGB surface encodes on write, so
/// the plain sample is an identity round-trip (unlike the RT
/// path's pre-2h blit, whose source held raw values in an
/// unorm texture and needed the pow-2.2 compensation — that
/// compensation now lives in the RT capture convention, not here).
pub struct PresentBlit {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl PresentBlit {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("present-blit-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("present-blit-shader"),
            source: wgpu::ShaderSource::Wgsl(BLIT_WGSL.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("present-blit-pipeline-layout"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("present-blit-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("present-blit-sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self {
            pipeline,
            layout,
            sampler,
        }
    }

    pub fn draw(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        src: &wgpu::Texture,
        dst_view: &wgpu::TextureView,
    ) {
        let src_view = src.create_view(&Default::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("present-blit-bg"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("present-blit-encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("present-blit-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: dst_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        queue.submit([encoder.finish()]);
    }
}
