//! Phase 4 gate tests (Part II §7), harness level.
//!
//! Gates 4.13 (end-to-end script) and 4.14 (soak) require the real
//! fork running in-game and are BLOCKED on Phase 0 tests 0.1/0.2 —
//! they are not faked here. GPU tests are adapter-conditional: with
//! no adapter they skip with a printed notice; on this VM lavapipe
//! provides a real (software) adapter and they do run.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use proto::{InputKind, MessageT, UiMode};
use ui_share::compositor::{composite_cpu_scaled, composite_headless, CompositeRequest};
use ui_share::frame::{apply_terrain_mask, apply_upload_plan, frame_notification, masked_tight_pixels, validate_frame, UploadPlan};
use ui_share::input::{key_to_input, mouse_button_to_input, text_to_input, EngineKey, InputRouter, Modifiers};
use ui_share::latency::LatencyReport;
use ui_share::picking::{pick_from_ui_click, PickCamera};
use ui_share::scaling::{compute_viewport, ScalingMode, Size2};
use ui_share::store::{DirtyRect, UiShmStore};
use ui_share::synthetic::{all_synthetic_frames, draw_gameplay_sidebar, draw_main_menu, image_diff, save_png, synthetic_terrain_rect};
use ui_share::{TerrainRect};

fn captures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("captures")
}

fn solid(w: u32, h: u32, rgba: [u8; 4]) -> Vec<u8> {
    let mut v = Vec::with_capacity((w * h * 4) as usize);
    for _ in 0..(w * h) { v.extend_from_slice(&rgba); }
    v
}

// ---------------------------------------------------------------- 4.1

#[test]
fn gate_4_1_synthetic_golden_frames_roundtrip_and_diff() {
    let (w, h) = (320u32, 200u32);
    let store = UiShmStore::create_memory(w, h).unwrap();
    let dir = captures_dir();
    for (name, canvas) in all_synthetic_frames(w, h) {
        let seq = store.write_frame_tight(&canvas.pixels, w, h, DirtyRect::full(w, h)).unwrap();
        let snap = store.read_latest().unwrap();
        assert_eq!(snap.seq, seq);
        assert_eq!(snap.tight_pixels(), canvas.pixels, "{name} store roundtrip must be byte-exact");
        // Self-diff is exactly zero; diff vs a different frame is not.
        let self_diff = image_diff(&snap.tight_pixels(), &canvas.pixels, 0);
        assert_eq!(self_diff.mismatched_pixels, 0, "{name}");
        save_png(&dir.join(format!("ui_{name}.png")), &snap.tight_pixels(), w, h).unwrap();
    }
    // Distinct frames actually differ (the goldens are not all one image).
    let a = draw_main_menu(w, h);
    let b = ui_share::synthetic::draw_inventory(w, h);
    let d = image_diff(&a.pixels, &b.pixels, 0);
    assert!(d.mismatch_fraction() > 0.05, "menu vs inventory should differ materially: {d:?}");
}

#[test]
fn gate_4_1_file_backed_store_roundtrip() {
    let path = std::env::temp_dir().join("ui_share_gate41.shm");
    let _ = std::fs::remove_file(&path);
    let (w, h) = (64u32, 48u32);
    let writer = UiShmStore::create_file(&path, w, h).unwrap();
    let px = solid(w, h, [12, 34, 56, 255]);
    writer.write_frame_tight(&px, w, h, DirtyRect::full(w, h)).unwrap();
    let reader = UiShmStore::open_file(&path).unwrap();
    let snap = reader.read_latest().unwrap();
    assert_eq!(snap.tight_pixels(), px);
    let _ = std::fs::remove_file(&path);
}

// ---------------------------------------------------------------- 4.2

#[test]
fn gate_4_2_terrain_rect_masking() {
    let (w, h) = (320u32, 200u32);
    let tr = synthetic_terrain_rect(w, h);
    let canvas = draw_gameplay_sidebar(w, h);
    // The synthetic gameplay frame already leaves the terrain rect
    // transparent; masking an *opaque-everywhere* variant proves the
    // mask itself (not the drawing) creates the hole, using the rect
    // reported in the UiFrame — here a non-default offset rect.
    let mut px = solid(w, h, [200, 30, 30, 255]);
    let offset_rect = TerrainRect { x: 10, y: 20, w: 100, h: 80 };
    apply_terrain_mask(&mut px, w, h, w * 4, Some(offset_rect));
    for y in 0..h { for x in 0..w {
        let a = px[((y * w + x) * 4 + 3) as usize];
        if offset_rect.contains(x as i32, y as i32) { assert_eq!(a, 0, "inside terrain rect must be transparent"); }
        else { assert_eq!(a, 255, "outside terrain rect stays as drawn"); }
    }}
    // And the real gameplay frame: sidebar opaque, terrain hole clear.
    let masked = masked_tight_pixels(
        &{ let s = UiShmStore::create_memory(w, h).unwrap(); s.write_frame_tight(&canvas.pixels, w, h, DirtyRect::full(w, h)).unwrap(); s.read_latest().unwrap() },
        Some(tr),
    );
    assert_eq!(masked[3], 0, "terrain origin alpha");
    let sidebar_x = tr.w as u32 + 5;
    assert_eq!(masked[((10 * w + sidebar_x) * 4 + 3) as usize], 255, "sidebar stays opaque");
}

// ---------------------------------------------------------------- 4.3 / 4.4

#[test]
fn gate_4_3_torn_frame_detection_under_concurrent_write() {
    let (w, h) = (128u32, 96u32);
    let store = UiShmStore::create_memory(w, h).unwrap();
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let writer_store = store.clone();
    let stop_w = stop.clone();
    let writer = std::thread::spawn(move || {
        let mut seq = 0u64;
        while !stop_w.load(std::sync::atomic::Ordering::Relaxed) {
            seq += 1;
            let pattern = (seq % 250 + 1) as u8;
            let px = vec![pattern; (w * h * 4) as usize];
            let _ = writer_store.write_frame_tight(&px, w, h, DirtyRect::full(w, h));
        }
        seq
    });
    // Reader samples continuously for ~2 s (the gate's 10 min form
    // needs the in-game writer; the mechanism is identical).
    let deadline = Instant::now() + Duration::from_millis(2000);
    let (mut reads, mut rejected, mut torn) = (0u64, 0u64, 0u64);
    while Instant::now() < deadline {
        match store.read_latest() {
            Ok(snap) => {
                reads += 1;
                let px = snap.tight_pixels();
                if px.iter().any(|&b| b != px[0]) { torn += 1; }
            }
            Err(_) => rejected += 1,
        }
    }
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    let written = writer.join().unwrap();
    println!("gate 4.3 harness: writer seqs={written}, reader ok={reads}, rejected={rejected}, torn={torn}");
    assert!(reads > 100, "reader should sample many frames, got {reads}");
    assert_eq!(torn, 0, "reader must never observe a torn frame");
}

#[test]
fn gate_4_3_checksum_catches_corruption() {
    let (w, h) = (64u32, 64u32);
    let store = UiShmStore::create_memory(w, h).unwrap();
    store.write_frame_tight(&solid(w, h, [9, 9, 9, 255]), w, h, DirtyRect::full(w, h)).unwrap();
    let snap = store.read_latest().unwrap();
    let off = store.debug_slot_pixel_offset(snap.buffer_index) + 40;
    store.debug_corrupt_byte(off, 0xAB).unwrap();
    // Corrupted active slot: retries cannot fix it — must error.
    assert!(store.read_latest().is_err(), "corrupted frame must be rejected, not returned");
}

#[test]
fn gate_4_4_reader_sees_complete_frame_while_writer_paused_mid_write() {
    let (w, h) = (64u32, 64u32);
    let store = UiShmStore::create_memory(w, h).unwrap();
    let frame_a = solid(w, h, [10, 0, 0, 255]);
    store.write_frame_tight(&frame_a, w, h, DirtyRect::full(w, h)).unwrap();
    // Simulate a writer paused mid-write: scribble half of frame B
    // into the *inactive* slot directly. The published frame is
    // still A and the reader must keep seeing exactly A.
    let hdr = store.read_header().unwrap();
    let inactive = 1 - hdr.buffer_index;
    let off = store.debug_slot_pixel_offset(inactive);
    for i in 0..(w * h * 2) as usize { store.debug_corrupt_byte(off + i, 77).unwrap(); }
    let snap = store.read_latest().unwrap();
    assert_eq!(snap.tight_pixels(), frame_a);
    // Writer completes: reader now sees complete frame B.
    let frame_b = solid(w, h, [20, 0, 0, 255]);
    store.write_frame_tight(&frame_b, w, h, DirtyRect::full(w, h)).unwrap();
    assert_eq!(store.read_latest().unwrap().tight_pixels(), frame_b);
    // Alternation: buffer index flips every publish.
    let s1 = store.read_latest().unwrap();
    store.write_frame_tight(&frame_a, w, h, DirtyRect::full(w, h)).unwrap();
    let s2 = store.read_latest().unwrap();
    assert_ne!(s1.buffer_index, s2.buffer_index);
    assert_eq!(s2.seq, s1.seq + 1);
}

// ---------------------------------------------------------------- 4.5

#[test]
fn gate_4_5_dirty_rect_cpu_matches_full_upload() {
    let (w, h) = (120u32, 80u32);
    let mut prev = solid(w, h, [30, 30, 30, 255]);
    let mut next = prev.clone();
    // Change a small region only (a moved cursor block).
    let dirty = DirtyRect { x: 40, y: 30, w: 12, h: 10 };
    for y in dirty.y..dirty.y + dirty.h { for x in dirty.x..dirty.x + dirty.w {
        let o = ((y as u32 * w + x as u32) * 4) as usize;
        next[o..o + 4].copy_from_slice(&[250, 240, 10, 255]);
    }}
    let plan = UploadPlan::for_frame(w, h, dirty);
    assert!(!plan.is_full);
    assert_eq!(plan.bytes, 12 * 10 * 4);
    let mut resident = prev.clone();
    apply_upload_plan(&mut resident, &next, w, h, &plan);
    assert_eq!(resident, next, "dirty-only upload must be pixel-identical to full upload");
    // Validation path: UiFrame notification must agree with the store.
    let store = UiShmStore::create_memory(w, h).unwrap();
    store.write_frame_tight(&prev, w, h, DirtyRect::full(w, h)).unwrap();
    let _ = &mut prev;
    store.write_frame_tight(&next, w, h, dirty).unwrap();
    let snap = store.read_latest().unwrap();
    let note = frame_notification(&snap, UiMode::MapInput, None);
    let validated = validate_frame(&store, &note).unwrap();
    assert_eq!(validated.dirty, dirty);
    // A stale notification (old seq) is rejected.
    let mut stale = frame_notification(&snap, UiMode::MapInput, None);
    stale.seq -= 1;
    assert!(validate_frame(&store, &stale).is_err());
}

// ---------------------------------------------------------------- 4.7 / 4.8

#[test]
fn gate_4_7_key_matrix_and_repeat() {
    let plain = Modifiers::default();
    let cases: Vec<(EngineKey, i32)> = vec![
        (EngineKey::Char('a'), 'a' as i32), (EngineKey::Char('z'), 'z' as i32),
        (EngineKey::Char('A'), 'A' as i32), (EngineKey::Char('0'), '0' as i32),
        (EngineKey::Char('9'), '9' as i32), (EngineKey::Char('!'), '!' as i32),
        (EngineKey::Char('/'), '/' as i32), (EngineKey::Char('?'), '?' as i32),
        (EngineKey::Up, ui_share::input::KEY_UP), (EngineKey::Down, ui_share::input::KEY_DOWN),
        (EngineKey::Left, ui_share::input::KEY_LEFT), (EngineKey::Right, ui_share::input::KEY_RIGHT),
        (EngineKey::Function(5), ui_share::input::KEY_F1 + 4),
        (EngineKey::Escape, ui_share::input::KEY_ESCAPE), (EngineKey::Enter, ui_share::input::KEY_ENTER),
        (EngineKey::Tab, ui_share::input::KEY_TAB), (EngineKey::Backspace, ui_share::input::KEY_BACKSPACE),
    ];
    for (key, code) in cases {
        let down = key_to_input(key, plain, true);
        assert_eq!(down.key_code, code, "{key:?}");
        assert!(down.pressed && down.kind == InputKind::Key);
        // Key repeat = another pressed event; release flips `pressed`.
        let repeat = key_to_input(key, plain, true);
        assert_eq!(repeat, down);
        let up = key_to_input(key, plain, false);
        assert!(!up.pressed && up.key_code == code);
        // Wire round-trip through the real FlatBuffers envelope.
        let bytes = proto::encode_message(&MessageT::Input(Box::new(down)));
        match proto::decode_message(&bytes).unwrap() {
            MessageT::Input(i) => assert_eq!(i.key_code, code),
            other => panic!("wrong message: {other:?}"),
        }
    }
    // Shift-modified letter carries both the code and the bitmask.
    let shifted = key_to_input(EngineKey::Char('A'), Modifiers { shift: true, ..Default::default() }, true);
    assert_eq!(shifted.modifiers, 1);
}

#[test]
fn gate_4_8_text_input_wire_roundtrip() {
    for text in ["Joey", "café", "☃ snowman", "日本語の名前", "a/b? [x]"] {
        let input = text_to_input(text);
        let bytes = proto::encode_message(&MessageT::Input(Box::new(input)));
        match proto::decode_message(&bytes).unwrap() {
            MessageT::Input(i) => assert_eq!(i.text.as_deref(), Some(text)),
            other => panic!("wrong message: {other:?}"),
        }
    }
}

// ---------------------------------------------------------------- 4.9 / 4.10

#[test]
fn gate_4_9_focus_and_mode_routing() {
    let tr = TerrainRect { x: 0, y: 0, w: 200, h: 200 };
    let ui_router = InputRouter::new(UiMode::UiInput, Some(tr));
    assert!(!ui_router.camera_drag_allowed());
    // In UiInput mode a click inside the terrain rect is still a UI click.
    match ui_router.route_ui_click(50, 50, 0, true, Modifiers::default()) {
        ui_share::input::ClickRoute::UiClick(i) => { assert_eq!((i.mouse_x, i.mouse_y), (50, 50)); }
        other => panic!("expected UiClick, got {other:?}"),
    }
    let map_router = InputRouter::new(UiMode::MapInput, Some(tr));
    assert!(map_router.camera_drag_allowed());
}

#[test]
fn gate_4_10_window_click_lands_on_menu_entry() {
    // Main-menu frame at 320x200 stretched into a 640x400 window:
    // a window click at 2x the UI coords of menu row 0 must route
    // to that row's UI coordinates (gate 4.10: UI clicks stay in
    // display-buffer coordinates after conversion).
    let ui_size = Size2 { w: 320, h: 200 };
    let vp = compute_viewport(Size2 { w: 640, h: 400 }, ui_size, ScalingMode::StretchNearest);
    let router = InputRouter::new(UiMode::UiInput, None);
    // Menu box starts at (80,50); first entry line at ~(112,120) UI px.
    match router.route_window_click(224.0, 240.0, &vp, ui_size, 0, true, Modifiers::default()) {
        ui_share::input::ClickRoute::UiClick(i) => assert_eq!((i.mouse_x, i.mouse_y), (112, 120)),
        other => panic!("expected UiClick, got {other:?}"),
    }
    // Direct mouse mapping sanity.
    let m = mouse_button_to_input(112, 120, 0, true, Modifiers::default());
    assert_eq!(m.kind, InputKind::MouseButton);
    assert_eq!(m.button, 0);
}

// ---------------------------------------------------------------- 4.11

#[test]
fn gate_4_11_picking_calibration_scene() {
    use glam::Vec3;
    let tr = TerrainRect { x: 0, y: 0, w: 400, h: 300 };
    let router = InputRouter::new(UiMode::MapInput, Some(tr));
    // Calibration: three occupied tiles; camera targets each in turn
    // and the terrain-rect center click must pick exactly it.
    for tile in [(5, 5, 0), (8, 3, 0), (2, 9, 1)] {
        let cam = PickCamera {
            target: Vec3::new(tile.0 as f32 + 0.5, tile.2 as f32 * 3.0 + 1.5, -(tile.1 as f32 + 0.5)),
            ..Default::default()
        };
        let res = pick_from_ui_click(7, &router, &cam, 200, 150, |x, y, z| (x, y, z) == tile)
            .expect("terrain click in MapInput mode must produce a PickResult");
        assert!(res.hit, "tile {tile:?}");
        let pos = res.pos.unwrap();
        assert_eq!((pos.x, pos.y, pos.z), tile);
        assert_eq!(res.request_id, 7);
    }
    // Empty scene: a result with hit=false, not a panic or a wrong tile.
    let cam = PickCamera::default();
    let res = pick_from_ui_click(8, &router, &cam, 200, 150, |_, _, _| false).unwrap();
    assert!(!res.hit);
}

// ---------------------------------------------------------------- GPU gates

fn gpu_or_skip(test_name: &str) -> bool {
    if ui_share::compositor::adapter_available() { true } else {
        println!("{test_name}: SKIPPED (no wgpu adapter) — vacuous, not a pass");
        false
    }
}

#[test]
fn gate_4_2_gpu_composite_shows_background_through_terrain_rect() {
    if !gpu_or_skip("gate_4_2_gpu") { return; }
    let (w, h) = (320u32, 200u32);
    let canvas = draw_gameplay_sidebar(w, h);
    let background = solid(w, h, [40, 90, 40, 255]); // the "3D world"
    let out = composite_headless(&CompositeRequest {
        background_tight: &background, ui_tight: &canvas.pixels,
        ui_size: (w, h), out_size: (w, h), scaling: ScalingMode::StretchNearest,
        dirty_plan: None, stale_ui_tight: None,
    }).unwrap();
    println!("gate 4.2 GPU adapter: {}", out.adapter_name);
    let img = out.image.as_raw();
    // Terrain hole (10,10): background green shows through.
    let o = ((10 * w + 10) * 4) as usize;
    assert!((img[o] as i32 - 40).abs() <= 2 && (img[o + 1] as i32 - 90).abs() <= 2, "terrain pixel {:?}", &img[o..o + 4]);
    // Sidebar (300,10): panel background, not world green.
    let o2 = ((10 * w + 300) * 4) as usize;
    assert_eq!(&img[o2..o2 + 3], &[24, 24, 32], "sidebar pixel");
    save_png(&captures_dir().join("ui_composited_gameplay.png"), img, w, h).unwrap();
}

#[test]
fn gate_4_5_gpu_dirty_upload_matches_full() {
    if !gpu_or_skip("gate_4_5_gpu") { return; }
    let (w, h) = (160u32, 120u32);
    let frame_a = draw_main_menu(w, h).pixels;
    let mut frame_b = frame_a.clone();
    let dirty = DirtyRect { x: 30, y: 40, w: 50, h: 30 };
    for y in dirty.y..dirty.y + dirty.h { for x in dirty.x..dirty.x + dirty.w {
        let o = ((y as u32 * w + x as u32) * 4) as usize;
        frame_b[o..o + 4].copy_from_slice(&[255, 128, 0, 255]);
    }}
    let background = solid(w, h, [10, 10, 60, 255]);
    let full = composite_headless(&CompositeRequest {
        background_tight: &background, ui_tight: &frame_b, ui_size: (w, h), out_size: (w, h),
        scaling: ScalingMode::StretchNearest, dirty_plan: None, stale_ui_tight: None,
    }).unwrap();
    let dirty_out = composite_headless(&CompositeRequest {
        background_tight: &background, ui_tight: &frame_b, ui_size: (w, h), out_size: (w, h),
        scaling: ScalingMode::StretchNearest,
        dirty_plan: Some(UploadPlan::for_frame(w, h, dirty)), stale_ui_tight: Some(&frame_a),
    }).unwrap();
    let d = image_diff(full.image.as_raw(), dirty_out.image.as_raw(), 0);
    assert_eq!(d.mismatched_pixels, 0, "GPU dirty-rect upload must equal full upload: {d:?}");
    // And the full GPU result matches the CPU oracle (opaque frame).
    let oracle = composite_cpu_scaled(&background, w, h, &frame_b, w, h, w, h, ScalingMode::StretchNearest);
    let d2 = image_diff(full.image.as_raw(), &oracle, 2);
    assert!(d2.passes(0.001), "GPU vs CPU oracle: {d2:?}");
}

#[test]
fn gate_4_6_gpu_resize_and_integer_scaling() {
    if !gpu_or_skip("gate_4_6_gpu") { return; }
    let (w, h) = (160u32, 100u32);
    let ui = draw_main_menu(w, h).pixels;
    for (out_w, out_h, mode) in [
        (160, 100, ScalingMode::StretchNearest),
        (320, 200, ScalingMode::StretchNearest),
        (400, 300, ScalingMode::IntegerScale), // 2x, letterboxed
        (200, 150, ScalingMode::StretchNearest),
    ] {
        let background = solid(out_w, out_h, [15, 15, 15, 255]);
        let out = composite_headless(&CompositeRequest {
            background_tight: &background, ui_tight: &ui, ui_size: (w, h), out_size: (out_w, out_h),
            scaling: mode, dirty_plan: None, stale_ui_tight: None,
        }).unwrap();
        assert_eq!(out.image.dimensions(), (out_w, out_h));
        let oracle = composite_cpu_scaled(&background, out_w, out_h, &ui, w, h, out_w, out_h, mode);
        let d = image_diff(out.image.as_raw(), &oracle, 2);
        assert!(d.passes(0.002), "resize {out_w}x{out_h} {mode:?}: {d:?}");
        if mode == ScalingMode::IntegerScale {
            // Letterbox corner stays background.
            assert_eq!(&out.image.as_raw()[0..3], &[15, 15, 15]);
        }
    }
}

#[test]
fn gate_4_12_latency_harness_segment() {
    if !gpu_or_skip("gate_4_12") { return; }
    let (w, h) = (320u32, 200u32);
    let store = UiShmStore::create_memory(w, h).unwrap();
    let canvas = draw_gameplay_sidebar(w, h);
    let background = solid(w, h, [40, 90, 40, 255]);
    let mut samples = Vec::new();
    let mut adapter = String::new();
    for _ in 0..5 {
        let t0 = Instant::now();
        store.write_frame_tight(&canvas.pixels, w, h, DirtyRect::full(w, h)).unwrap();
        let snap = store.read_latest().unwrap();
        let read_ms = t0.elapsed().as_secs_f64() * 1000.0;
        let masked = masked_tight_pixels(&snap, Some(synthetic_terrain_rect(w, h)));
        let out = composite_headless(&CompositeRequest {
            background_tight: &background, ui_tight: &masked, ui_size: (w, h), out_size: (w, h),
            scaling: ScalingMode::StretchNearest, dirty_plan: None, stale_ui_tight: None,
        }).unwrap();
        adapter = out.adapter_name.clone();
        let total = read_ms + out.stats.upload.as_secs_f64() * 1000.0 + out.stats.composite_and_readback.as_secs_f64() * 1000.0;
        samples.push(total);
    }
    let report = LatencyReport::from_samples(samples, &adapter);
    println!("gate 4.12 harness segment [{}]: median {:.1} ms, p99 {:.1} ms, samples {:?}", report.label, report.median_ms, report.p99_ms, report.samples_ms);
    // The harness segment includes full device creation per frame on
    // llvmpipe; we record the numbers and only sanity-bound them.
    // The gate's 50/100 ms budgets apply to the in-game end-to-end
    // path (blocked) — see docs/phase4-ui.md.
    assert!(report.median_ms < 30_000.0, "harness segment should complete in seconds at worst");
}
