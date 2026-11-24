//! S4 gate, T2 engine half: the engine consumes the sim's UI
//! frames through the exact path the live loop uses —
//! `engine::frame::prepare_ui` (validate_frame + read_latest +
//! terrain masking over the sim's own store) — and the click
//! separation the protocol defines (ui-share's InputRouter +
//! picking, gate 4.11): in MapInput a click inside the sim's
//! reported terrain rect is a world pick against the sim's
//! world; in UiInput no click is ever a world pick.
//!
//! The sim side of the same flows (routing, PickResult
//! recording, resize) is gated in crates/sim/tests/ui_gate.rs.

use std::path::PathBuf;

use engine::frame::prepare_ui;
use proto::{InputKind, InputT, MessageT, UiMode};
use sim::{DefRegistry, Scene, World};
use ui_share::input::{ClickRoute, InputRouter, Modifiers};
use ui_share::picking::{pick_from_ui_click, PickCamera};
use ui_share::synthetic::synthetic_terrain_rect;

fn defs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/sim-defs/generated/defs.json")
}

fn scene_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../sim/scenes/{name}.toml"))
}

fn build(scene_name: &str) -> (World, DefRegistry) {
    let reg = DefRegistry::load(&defs_path()).expect("registry loads");
    let scene = Scene::load_file(&scene_path(scene_name)).expect("scene loads");
    let mut world = World::from_scene(&reg, &scene).expect("world builds");
    world.snapshot(&reg);
    (world, reg)
}

fn key_input(c: char) -> InputT {
    let mut input = InputT::default();
    input.kind = InputKind::Key;
    input.key_code = c as i32;
    input.pressed = true;
    input
}

#[test]
fn t2_engine_prepare_ui_consumes_sim_frames() {
    let (mut world, reg) = build("street_block");

    // Gameplay frame through prepare_ui: mode MapInput, the
    // terrain rect follows the layout rule, and the masked
    // tight pixels have alpha 0 across the whole terrain rect
    // (the sim leaves the terrain window unpainted, exactly
    // like the real game's Route B frames).
    let note = world.refresh_ui(&reg).expect("gameplay frame");
    let prepared = prepare_ui(world.ui_store(), &note).expect("prepare_ui accepts the sim frame");
    assert_eq!(prepared.mode, UiMode::MapInput);
    assert_eq!(prepared.size, (1280, 720));
    let expect = synthetic_terrain_rect(1280, 720);
    let rect = prepared.terrain_rect.expect("gameplay frame has a terrain rect");
    assert_eq!((rect.x, rect.y, rect.w, rect.h), (expect.x, expect.y, expect.w, expect.h));
    let stride = (prepared.size.0 * 4) as usize;
    for y in rect.y..rect.y + rect.h {
        for x in rect.x..rect.x + rect.w {
            let a = prepared.tight[(y as usize * stride) + (x as usize * 4) + 3];
            assert_eq!(a, 0, "terrain rect pixel ({x},{y}) must be masked/transparent");
        }
    }
    // The sidebar is painted (non-transparent) somewhere right
    // of the terrain rect.
    let sx = (rect.x + rect.w + 8) as usize;
    let mut painted = false;
    for y in 0..prepared.size.1 as usize {
        if prepared.tight[y * stride + sx * 4 + 3] > 0 {
            painted = true;
            break;
        }
    }
    assert!(painted, "sidebar column must carry painted pixels");

    // Inventory frame through prepare_ui: UiInput, no terrain
    // rect, and real text pixels present (the frame is not a
    // blank/placeholder panel).
    let msgs = world.apply_input(&reg, &key_input('i'));
    let note = match &msgs[0] {
        MessageT::UiFrame(f) => (**f).clone(),
        other => panic!("expected UiFrame, got {other:?}"),
    };
    let prepared = prepare_ui(world.ui_store(), &note).expect("prepare_ui accepts inventory");
    assert_eq!(prepared.mode, UiMode::UiInput);
    assert!(prepared.terrain_rect.is_none(), "UiInput frames mask nothing");
    let painted_px = prepared.tight.iter().skip(3).step_by(4).filter(|&&a| a > 0).count();
    assert!(painted_px > 10_000, "inventory frame carries real rendered content ({painted_px} px)");
}

#[test]
fn t2_engine_click_separation_and_pick_flow() {
    let (mut world, reg) = build("street_block");
    let note = world.refresh_ui(&reg).expect("gameplay frame");
    let prepared = prepare_ui(world.ui_store(), &note).expect("prepare_ui");
    let rect = prepared.terrain_rect.expect("terrain rect");

    // MapInput router over the sim's reported geometry.
    let router = InputRouter::new(prepared.mode, prepared.terrain_rect);
    let (cx, cy) = (rect.x + rect.w / 2, rect.y + rect.h / 2);
    assert!(router.is_world_pick(cx, cy), "terrain centre is a world pick");
    match router.route_ui_click(cx, cy, 0, true, Modifiers::default()) {
        ClickRoute::WorldPick { ui_x, ui_y } => assert_eq!((ui_x, ui_y), (cx, cy)),
        other => panic!("expected WorldPick, got {other:?}"),
    }
    // A sidebar click is a UI click, never a pick.
    let sx = rect.x + rect.w + 20;
    assert!(!router.is_world_pick(sx, cy));
    assert!(matches!(
        router.route_ui_click(sx, cy, 0, true, Modifiers::default()),
        ClickRoute::UiClick(_)
    ));

    // The pick itself, run the engine's way against the sim's
    // world occupancy: a PickResult comes back, and the sim
    // records it without changing.
    let cam = PickCamera {
        target: glam::Vec3::new(5.0, 0.0, -5.0),
        yaw_deg: 0.0,
        pitch_deg: 45.0,
        distance: 28.0,
        fov_y_deg: 60.0,
    };
    let pick = pick_from_ui_click(42, &router, &cam, cx, cy, |x, y, z| {
        world.terrain_at([x, y, z]).is_some_and(|t| t != 0)
    })
    .expect("terrain click produces a PickResult");
    assert_eq!(pick.request_id, 42);
    if pick.hit {
        let p = pick.pos.as_ref().expect("hit carries a pos");
        assert!(
            (0..24).contains(&p.x) && (0..24).contains(&p.y) && p.z == 0,
            "hit tile lies in the street_block scene: {p:?}"
        );
    }
    let seq_before = world.seq;
    world.apply_pick_result(&pick);
    assert_eq!(world.last_pick().map(|p| p.request_id), Some(42));
    assert_eq!(world.seq, seq_before);

    // UiInput: the same terrain-centre click is NOT a world
    // pick — the router says UiClick and picking returns None.
    let msgs = world.apply_input(&reg, &key_input('i'));
    let note = match &msgs[0] {
        MessageT::UiFrame(f) => (**f).clone(),
        other => panic!("expected UiFrame, got {other:?}"),
    };
    let prepared = prepare_ui(world.ui_store(), &note).expect("prepare_ui");
    let router = InputRouter::new(prepared.mode, prepared.terrain_rect);
    assert!(!router.is_world_pick(cx, cy), "UiInput: no world picks anywhere");
    assert!(matches!(
        router.route_ui_click(cx, cy, 0, true, Modifiers::default()),
        ClickRoute::UiClick(_)
    ));
    assert!(
        pick_from_ui_click(43, &router, &cam, cx, cy, |_, _, _| true).is_none(),
        "UiInput click must not produce a PickResult"
    );
}
