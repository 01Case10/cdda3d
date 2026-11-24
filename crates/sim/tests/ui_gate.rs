//! S4 gate, T2 — open the UI (plan §3.3 + test table T2).
//!
//! Stage 1 (failing-first): these tests use only the pre-S4
//! public surface (`World::apply_input`, `World::player_pos`,
//! proto types) and assert the S4 contract — they were written
//! before the UI existed and run against the S3 code, where
//! they fail (recorded in the S4 slice record): pre-S4, `i` is
//! an unbound key that consumes a no-op turn (a Delta comes
//! back, no UiFrame), and a movement key afterwards moves the
//! player, because there is no UiInput mode to consume it.
//!
//! Stage 2 (added with the implementation): the cell-grid text
//! honesty checks, the ui-share reader run against sim-written
//! frames, terrain-rect masking across Resize, mouse routing,
//! and the torn-frame reader race — those need the S4 API
//! (`World::ui_grid`, `World::ui_store`, `World::apply_resize`,
//! `World::apply_pick_result`) and live below the stage-1 tests.

use std::path::PathBuf;

use proto::{InputKind, InputT, MessageT, UiFrameT, UiMode};
use sim::{DefRegistry, Scene, World};

fn defs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/sim-defs/generated/defs.json")
}

fn scene_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("scenes/{name}.toml"))
}

fn key_input(c: char) -> InputT {
    let mut input = InputT::default();
    input.kind = InputKind::Key;
    input.key_code = c as i32;
    input.pressed = true;
    input
}

/// The engine's Escape key code (ui-share `KEY_ESCAPE`): Esc is
/// not a Unicode scalar, it arrives as a Key input with this
/// platform-independent code.
const KEY_ESCAPE: i32 = 0x1001;

fn esc_input() -> InputT {
    let mut input = InputT::default();
    input.kind = InputKind::Key;
    input.key_code = KEY_ESCAPE;
    input.pressed = true;
    input
}

fn build(scene_name: &str) -> (World, DefRegistry) {
    let reg = DefRegistry::load(&defs_path()).expect("registry loads");
    let scene = Scene::load_file(&scene_path(scene_name)).expect("scene loads");
    let mut world = World::from_scene(&reg, &scene).expect("world builds");
    world.snapshot(&reg);
    (world, reg)
}

fn ui_frames(msgs: &[MessageT]) -> Vec<&UiFrameT> {
    msgs.iter()
        .filter_map(|m| match m {
            MessageT::UiFrame(f) => Some(&**f),
            _ => None,
        })
        .collect()
}

fn has_delta(msgs: &[MessageT]) -> bool {
    msgs.iter().any(|m| matches!(m, MessageT::Delta(_)))
}

#[test]
fn t2_i_opens_inventory_uiframe() {
    let (mut world, reg) = build("street_block");
    let msgs = world.apply_input(&reg, &key_input('i'));
    let frames = ui_frames(&msgs);
    assert_eq!(
        frames.len(),
        1,
        "'i' must produce exactly one UiFrame, got {msgs:?}"
    );
    assert_eq!(
        frames[0].mode,
        UiMode::UiInput,
        "inventory frame must flip the mode to UiInput"
    );
    assert!(
        !has_delta(&msgs),
        "opening the UI is not a world turn: no Delta may be emitted"
    );
}

#[test]
fn t2_movement_key_in_uiinput_moves_cursor_not_player() {
    let (mut world, reg) = build("street_block");
    let start = world.player_pos().expect("street_block has a player");
    world.apply_input(&reg, &key_input('i'));
    // 'l' is east in MapInput. In UiInput it must be consumed by
    // the UI: a new UiFrame (cursor/frame change), no Delta, and
    // the player does not move.
    let msgs = world.apply_input(&reg, &key_input('l'));
    assert_eq!(
        world.player_pos(),
        Some(start),
        "player must not move while in UiInput"
    );
    assert!(
        !has_delta(&msgs),
        "a UI-consumed key must not emit a Delta: {msgs:?}"
    );
    assert_eq!(
        ui_frames(&msgs).len(),
        1,
        "a cursor move publishes exactly one UiFrame: {msgs:?}"
    );
}

fn mouse_input(x: i32, y: i32) -> InputT {
    let mut input = InputT::default();
    input.kind = InputKind::MouseButton;
    input.mouse_x = x;
    input.mouse_y = y;
    input.pressed = true;
    input
}

#[test]
fn t2_esc_returns_to_mapinput() {
    let (mut world, reg) = build("street_block");
    world.apply_input(&reg, &key_input('i'));
    let msgs = world.apply_input(&reg, &esc_input());
    let frames = ui_frames(&msgs);
    assert_eq!(frames.len(), 1, "Esc must publish one UiFrame: {msgs:?}");
    assert_eq!(
        frames[0].mode,
        UiMode::MapInput,
        "Esc must flip the mode back to MapInput"
    );
    // And the world is live again: a movement key is a turn.
    let msgs = world.apply_input(&reg, &key_input('l'));
    assert!(has_delta(&msgs), "after Esc, movement is a world turn again");
}

// ---------------------------------------------------------------------
// Stage 2 (written with the S4 implementation): text honesty,
// the ui-share reader run against sim-written frames, terrain-
// rect masking across Resize, mouse routing + pick recording,
// and the torn-frame reader race.
// ---------------------------------------------------------------------

use proto::{PickResultT, Vec3iT};
use sim::ui::{Ui, UiState, UiView};
use ui_share::frame::validate_frame;
use ui_share::synthetic::synthetic_terrain_rect;
use ui_share::store::UiShmStore;

#[test]
fn t2_crafting_and_lookaround_roundtrip() {
    let (mut world, reg) = build("street_block");
    let start = world.player_pos().expect("player");

    // '&' opens Crafting (UiInput), Esc returns to Gameplay.
    let msgs = world.apply_input(&reg, &key_input('&'));
    let frames = ui_frames(&msgs);
    assert_eq!(frames.len(), 1, "one UiFrame for '&': {msgs:?}");
    assert_eq!(frames[0].mode, UiMode::UiInput);
    assert_eq!(world.ui_state(), UiState::Crafting);
    assert!(!has_delta(&msgs));
    let msgs = world.apply_input(&reg, &esc_input());
    assert_eq!(ui_frames(&msgs)[0].mode, UiMode::MapInput);
    assert_eq!(world.ui_state(), UiState::Gameplay);

    // ';' opens LookAround with the cursor on the player; 'l'
    // moves the LOOK cursor east — the player does not move and
    // no turn is consumed.
    let msgs = world.apply_input(&reg, &key_input(';'));
    assert_eq!(ui_frames(&msgs)[0].mode, UiMode::UiInput);
    assert_eq!(world.ui_state(), UiState::LookAround);
    assert_eq!(world.ui.look_cursor(), start);
    let msgs = world.apply_input(&reg, &key_input('l'));
    assert_eq!(ui_frames(&msgs).len(), 1, "look move publishes: {msgs:?}");
    assert!(!has_delta(&msgs));
    assert_eq!(world.ui.look_cursor(), [start[0] + 1, start[1], start[2]]);
    assert_eq!(world.player_pos(), Some(start), "player must not move");

    // 'q' backs out too (§3.3).
    let msgs = world.apply_input(&reg, &key_input('q'));
    assert_eq!(ui_frames(&msgs)[0].mode, UiMode::MapInput);
    assert_eq!(world.ui_state(), UiState::Gameplay);
}

#[test]
fn t2_inventory_grid_spells_carried_items() {
    let (mut world, reg) = build("street_block");
    // The gameplay grid (initial publish) spells the sidebar.
    let rows = world.ui_grid().row_strings();
    assert!(
        rows.iter().any(|r| r.contains("Cataclysm: Sim")),
        "gameplay grid must spell the sidebar title:\n{}",
        rows.join("\n")
    );
    world.apply_input(&reg, &key_input('i'));
    // Text honesty at the cell-grid level: the rendered grid's
    // characters spell the scene's carried list — "rock × 3"
    // and "stick" (street_block [[carried]], plan §3.3).
    let rows = world.ui_grid().row_strings();
    assert!(
        rows.iter().any(|r| r.contains("rock") && r.contains("× 3")),
        "inventory grid must spell 'rock × 3':\n{}",
        rows.join("\n")
    );
    assert!(
        rows.iter().any(|r| r.contains("stick")),
        "inventory grid must spell 'stick':\n{}",
        rows.join("\n")
    );
}

#[test]
fn t2_dirty_rect_and_seq_progression() {
    let (mut world, reg) = build("street_block");
    let seq0 = world.ui_store().read_header().unwrap().seq;
    let msgs = world.apply_input(&reg, &key_input('i'));
    let open = ui_frames(&msgs)[0];
    assert_eq!(open.seq, seq0 + 1, "open publishes exactly one frame");
    // A cursor move republishes with a PARTIAL dirty rect (the
    // changed list rows), not a full frame.
    let msgs = world.apply_input(&reg, &key_input('j'));
    let cur = ui_frames(&msgs)[0];
    assert_eq!(cur.seq, open.seq + 1);
    let area = (cur.dirty_w as u64) * (cur.dirty_h as u64);
    let full = (cur.width as u64) * (cur.height as u64);
    assert!(
        area < full,
        "cursor-move dirty rect must be partial: {} of {} px",
        area,
        full
    );
    assert!(cur.dirty_w > 0 && cur.dirty_h > 0);
}

#[test]
fn t2_ui_share_reader_accepts_sim_frames() {
    let (mut world, reg) = build("street_block");
    // The initial frame (published at world build) reads back
    // through the real reader: seq + checksum verified inside
    // read_latest.
    let snap = world.ui_store().read_latest().expect("initial frame reads");
    assert_eq!(snap.seq, 1);
    assert_eq!((snap.width, snap.height), (1280, 720));
    assert_eq!(snap.pixels.len(), (snap.stride * snap.height) as usize);

    // Every frame the routing returns validates against the
    // store through ui-share's own validate_frame, and the
    // reader's latest snapshot agrees with the notification.
    for input in [key_input('i'), key_input('j'), esc_input(), key_input('&')] {
        let msgs = world.apply_input(&reg, &input);
        let frames = ui_frames(&msgs);
        assert_eq!(frames.len(), 1, "expected one UiFrame for {input:?}");
        let f = frames[0];
        validate_frame(world.ui_store(), f).expect("sim frame must validate");
        let snap = world.ui_store().read_latest().expect("frame reads back");
        assert_eq!(snap.seq, f.seq);
        assert_eq!(snap.buffer_index, f.buffer_index);
        assert_eq!(snap.dirty.x, f.dirty_x);
        assert_eq!(snap.dirty.w, f.dirty_w);
    }
}

#[test]
fn t2_terrain_rect_rule_and_resize() {
    let (mut world, reg) = build("street_block");
    // Gameplay: the UiFrame's terrain rect is the synthetic
    // layout rule at the default size.
    let f = world.refresh_ui(&reg).expect("gameplay frame");
    let t = synthetic_terrain_rect(1280, 720);
    assert_eq!(
        (f.terrain_rect_x, f.terrain_rect_y, f.terrain_rect_w, f.terrain_rect_h),
        (t.x, t.y, t.w, t.h),
        "terrain rect must follow the layout rule"
    );
    assert_eq!((f.width, f.height), (1280, 720));

    // UiInput frames carry no terrain rect (-1 per the
    // frame_notification convention).
    let msgs = world.apply_input(&reg, &key_input('i'));
    let f = ui_frames(&msgs)[0];
    assert_eq!(f.terrain_rect_w, -1, "UiInput frame: no terrain rect");
    world.apply_input(&reg, &esc_input());

    // Resize: one UiFrame with the new geometry, full-frame
    // dirty, terrain rect recomputed by the same rule.
    let msgs = world.apply_resize(&reg, 800, 600);
    let frames = ui_frames(&msgs);
    assert_eq!(frames.len(), 1, "resize publishes one UiFrame: {msgs:?}");
    let f = frames[0];
    assert_eq!((f.width, f.height), (800, 600));
    assert_eq!(f.stride, 800 * 4, "tight stride at the new size");
    assert_eq!(
        (f.dirty_x, f.dirty_y, f.dirty_w, f.dirty_h),
        (0, 0, 800, 600),
        "resize dirty rect is the full frame"
    );
    let t = synthetic_terrain_rect(800, 600);
    assert_eq!(
        (f.terrain_rect_x, f.terrain_rect_y, f.terrain_rect_w, f.terrain_rect_h),
        (t.x, t.y, t.w, t.h),
        "terrain rect recomputed by the rule after resize"
    );
    validate_frame(world.ui_store(), f).expect("resized frame validates");
    // Same-size resize is a no-op.
    assert!(world.apply_resize(&reg, 800, 600).is_empty());
}

#[test]
fn t2_mouse_routing_and_pick_recording() {
    let (mut world, reg) = build("street_block");
    // MapInput click: not a UI event, not a turn — the engine
    // owns terrain picking and answers with a PickResult
    // message; the sim produces nothing for the click itself.
    let msgs = world.apply_input(&reg, &mouse_input(100, 100));
    assert!(msgs.is_empty(), "MapInput click: no reply: {msgs:?}");
    assert!(world.last_pick().is_none());

    // The engine's PickResult is recorded verbatim and changes
    // nothing (no seq advance, no move).
    let mut pick = PickResultT::default();
    pick.request_id = 7;
    pick.hit = true;
    let mut pick_pos = Vec3iT::default();
    pick_pos.x = 5;
    pick_pos.y = 6;
    pick_pos.z = 0;
    pick.pos = Some(pick_pos);
    let seq_before = world.seq;
    let pos_before = world.player_pos();
    world.apply_pick_result(&pick);
    let recorded = world.last_pick().expect("pick recorded");
    assert_eq!(recorded.request_id, 7);
    assert!(recorded.hit);
    let rp = recorded.pos.as_ref().expect("pick pos present");
    assert_eq!((rp.x, rp.y, rp.z), (5, 6, 0));
    assert_eq!(world.seq, seq_before, "a pick is not a turn");
    assert_eq!(world.player_pos(), pos_before);

    // UiInput click on the second inventory row moves the
    // cursor there, publishes one UiFrame, and produces NO
    // pick (the recorded pick is untouched).
    world.apply_input(&reg, &key_input('i'));
    let layout = world.ui.layout();
    let x = (layout.list_left_col as u32 * layout.cell_w + 2) as i32;
    let y = ((layout.list_top_row + 1) as u32 * layout.cell_h + 2) as i32;
    let msgs = world.apply_input(&reg, &mouse_input(x, y));
    assert_eq!(ui_frames(&msgs).len(), 1, "UI click publishes: {msgs:?}");
    assert!(!has_delta(&msgs));
    assert_eq!(world.ui.cursor(), 1, "click selects the second row");
    assert_eq!(
        world.last_pick().map(|p| p.request_id),
        Some(7),
        "a UI click must not produce a world pick"
    );
}

#[test]
fn t2_main_menu_entry_via_ui_start() {
    // The MainMenu entry decision (plan §3.3 amendment): a
    // scene `ui_start` key (or the server's --ui-start flag) —
    // never Esc from gameplay.
    let reg = DefRegistry::load(&defs_path()).expect("registry loads");
    let mut scene = Scene::load_file(&scene_path("street_block")).expect("scene loads");
    scene.ui_start = Some("main-menu".to_string());
    let mut world = World::from_scene(&reg, &scene).expect("world builds");
    world.snapshot(&reg);
    assert_eq!(world.ui_state(), UiState::MainMenu);
    let f = world.refresh_ui(&reg).expect("menu frame");
    assert_eq!(f.mode, UiMode::UiInput, "menu is a UiInput state");
    let rows = world.ui_grid().row_strings();
    assert!(
        rows.iter().any(|r| r.contains("CATACLYSM")),
        "menu grid spells the title:\n{}",
        rows.join("\n")
    );
    assert!(rows.iter().any(|r| r.contains("New Game")));
    // Esc in the menu is a no-op (no game behind it yet).
    let msgs = world.apply_input(&reg, &esc_input());
    assert!(msgs.is_empty(), "Esc in MainMenu publishes nothing: {msgs:?}");
    assert_eq!(world.ui_state(), UiState::MainMenu);
}

#[test]
fn t2_torn_frame_race() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    // The real topology: the writer owns a file store; the
    // reader opens the SAME file through a second handle.
    let path = std::env::temp_dir().join(format!("sim-ui-race-{}.shm", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let mut ui = Ui::new_file(path.clone(), 640, 480).expect("writer store");
    ui.open(UiState::Inventory, Some([0, 0, 0]));
    let reader = UiShmStore::open_file(&path).expect("reader handle");

    const PUBLISHES: u64 = 300;
    let done = Arc::new(AtomicBool::new(false));
    let done_w = done.clone();
    let writer = std::thread::spawn(move || {
        for n in 0..PUBLISHES {
            let view = UiView {
                scene_name: "race".to_string(),
                player_pos: Some([0, 0, 0]),
                player_hp: Some(255),
                clock: format!("frame {n}"),
                weather: "clear".to_string(),
                carried: vec![("rock".to_string(), 3), ("stick".to_string(), 1)],
                messages: Vec::new(),
                look_lines: Vec::new(),
            };
            ui.publish(&view).expect("publish");
            if n % 3 == 0 {
                // Alternate content so frames genuinely differ.
                ui.apply_ui_key(b'j' as i32, &view, ([0, 0, 0], [24, 24, 0]));
            }
        }
        done_w.store(true, Ordering::SeqCst);
        ui
    });

    // Reader: every successful read_latest is checksum-verified
    // by the reader itself; seqs must strictly increase; the
    // final published seq must be observed.
    let mut last_seq = 0u64;
    let mut reads = 0u64;
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        match reader.read_latest() {
            Ok(snap) => {
                reads += 1;
                assert!(
                    snap.seq >= last_seq,
                    "seq must never go backwards across reads: {} after {}",
                    snap.seq,
                    last_seq
                );
                assert_eq!(snap.pixels.len(), (snap.stride * snap.height) as usize);
                last_seq = snap.seq;
                if snap.seq >= PUBLISHES {
                    break;
                }
            }
            Err(_) => {
                // A rejected read is the checksum discipline
                // working (mid-publish), never a torn frame
                // handed to the caller.
            }
        }
        assert!(Instant::now() < deadline, "reader never saw the final seq");
        if done.load(Ordering::SeqCst) && last_seq == 0 {
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    let ui = writer.join().expect("writer thread");
    assert!(reads > 0, "reader must observe frames");
    assert_eq!(last_seq, PUBLISHES, "final read is the last publish");
    // Quiescent store: the final notification validates.
    let snap = ui.store().read_latest().expect("final read");
    let note = ui_share::frame::frame_notification(
        &snap,
        UiMode::UiInput,
        None,
    );
    validate_frame(ui.store(), &note).expect("final frame validates");
    let _ = std::fs::remove_file(&path);
}
