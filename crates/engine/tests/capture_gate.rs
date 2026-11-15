//! Gate test 2.10 (adapter-conditional part): the headless capture mode
//! itself. On a machine WITH a wgpu adapter, the capture must produce a
//! PNG of the requested size that is not uniformly one color. On a
//! machine WITHOUT an adapter (the recon VM), the mode must fail with
//! the named NoAdapter error and write no file — that clean failure is
//! the pass condition there; the pixel comparison is not run.

use engine::capture::{run_capture, CaptureArgs, CaptureError};

#[test]
fn test_2_10_headless_capture_mode() {
    let out = std::env::temp_dir().join("cdda_voxel_capture_gate.png");
    let _ = std::fs::remove_file(&out);
    let args = CaptureArgs {
        size: (128, 96),
        out: out.clone(),
        ..CaptureArgs::default()
    };
    match run_capture(&args) {
        Err(CaptureError::NoAdapter) => {
            assert!(
                !out.exists(),
                "no PNG may be written when there is no adapter"
            );
            eprintln!("2.10: no wgpu adapter on this machine — clean NoAdapter failure verified; pixel comparison NOT RUN");
        }
        Err(e) => panic!("capture failed with an unexpected error: {e}"),
        Ok(outcome) => {
            assert_eq!(outcome.size, (128, 96));
            let img = image::open(&out).expect("capture PNG must be readable");
            assert_eq!(img.width(), 128);
            assert_eq!(img.height(), 96);
            assert!(
                outcome.distinct_colors > 1,
                "capture must not be uniformly one color"
            );
            eprintln!(
                "2.10: capture rendered {} quads, {} distinct colors",
                outcome.quads, outcome.distinct_colors
            );
        }
    }
}

#[test]
fn capture_rejects_unknown_snapshot_source() {
    let args = CaptureArgs {
        snapshot: "does-not-exist".to_string(),
        ..CaptureArgs::default()
    };
    match run_capture(&args) {
        Err(CaptureError::UnsupportedSnapshot(s)) => assert_eq!(s, "does-not-exist"),
        other => panic!("expected UnsupportedSnapshot, got {other:?}"),
    }
}
