//! S1 measurement gate: the Option C handshake cost, measured
//! over a real loopback socket against the actual server. The
//! numbers are printed (and recorded in tools/sim-server/README.md);
//! the assertions are sanity bounds only — timings vary by machine.

use std::path::PathBuf;

use sim::measure::measure;

fn defs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/sim-defs/generated/defs.json")
}

fn scene_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("scenes/{name}.toml"))
}

#[test]
fn measure_handshake_cost() {
    let m = measure(Some(&defs_path()), &scene_path("stub_house")).expect("measurement runs");
    eprintln!(
        "S1 MEASURE: defs={} registry_load={:?} tiledefs_frame_bytes={} snapshot_build={:?} connect_to_tiledefs={:?}",
        m.def_count, m.registry_load, m.tiledefs_frame_bytes, m.scene_snapshot_build,
        m.connect_to_tiledefs
    );
    assert_eq!(m.def_count, 14_262);
    assert!(
        m.tiledefs_frame_bytes > 1_000_000,
        "the full table is a multi-MB handshake, got {} bytes",
        m.tiledefs_frame_bytes
    );
}
