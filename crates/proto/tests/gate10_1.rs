//! Phase 10.1 gates (docs/phase10-sky-gi.md §8):
//!
//! 10.1a — EnvironmentUpdate round-trip: Rust encode/decode, the C++
//! golden pair (gate 1.3 pattern), and the old-reader test: a decoder
//! built from the FROZEN pre-10.1 schema (tests/fixtures/cdda_ipc_v1.fbs,
//! bindings in proto::generated_v1) must ignore environment messages
//! and environment fields without erroring.
//!
//! The exporter send-rule half (10.1b) lives in the C++ fork harness
//! (tools/fork-harness), where the exporter core runs.

use std::path::{Path, PathBuf};
use std::process::Command;

use proto::samples::*;
use proto::*;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn ensure_cpp_roundtrip() -> Option<PathBuf> {
    let dir = repo_root().join("tools/stub-server");
    let bin = dir.join("cpp-roundtrip");
    if bin.exists() {
        return Some(bin);
    }
    if Command::new("g++")
        .arg("--version")
        .output()
        .map(|o| !o.status.success())
        .unwrap_or(true)
    {
        eprintln!("SKIP: g++ not available, C++ golden check not run");
        return None;
    }
    let out = Command::new("sh")
        .arg("build.sh")
        .current_dir(&dir)
        .output()
        .expect("run build.sh");
    if !out.status.success() {
        eprintln!(
            "SKIP: stub build failed:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        return None;
    }
    bin.exists().then_some(bin)
}

fn tmpdir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cdda-proto-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

// ------------------------------------------------------------ 10.1a (Rust)
#[test]
fn t10_1a_environment_round_trip() {
    // Standalone message.
    let msg = MessageT::EnvironmentUpdate(Box::new(sample_environment()));
    assert_eq!(decode_message(&encode_message(&msg)).unwrap(), msg);

    // Piggybacked on Snapshot (sample_snapshot carries the same
    // environment) and on Delta when present.
    let snap = MessageT::Snapshot(Box::new(sample_snapshot()));
    assert_eq!(decode_message(&encode_message(&snap)).unwrap(), snap);
    let mut delta = sample_delta();
    delta.environment = Some(Box::new(sample_environment()));
    let dmsg = MessageT::Delta(Box::new(delta));
    assert_eq!(decode_message(&encode_message(&dmsg)).unwrap(), dmsg);

    // Per-tile sky exposure round-trips: SubmapBlock.outside and
    // TileChange.outside.
    let block = sample_house_block();
    let outside = block.outside.expect("sample block carries outside");
    assert_eq!(outside.len(), TILES);
    // The shared stub-house rule: interior floor tiles are not
    // sky-exposed, the rest are.
    assert!(!outside[idx(6, 6)], "house interior must not be sky-exposed");
    assert!(outside[idx(0, 0)], "corner/wall tile is sky-exposed");
    let delta = sample_delta();
    assert!(
        delta.tile_changes.unwrap()[0].outside,
        "door tile on the perimeter is sky-exposed"
    );
}

// ------------------------------------------------- 10.1a (C++ golden pair)
#[test]
fn t10_1a_cpp_golden_environment() {
    let Some(bin) = ensure_cpp_roundtrip() else {
        return;
    };
    let dir = tmpdir("envgolden");
    let out = Command::new(&bin)
        .args(["write", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "cpp write goldens failed");

    // C++ -> Rust: the standalone environment golden decodes to the
    // Rust sample, and the snapshot golden (now carrying the piggyback
    // environment + per-tile outside) matches sample_snapshot.
    let env = std::fs::read(dir.join("golden_environment.bin")).unwrap();
    assert_eq!(
        decode_message(&env).unwrap(),
        MessageT::EnvironmentUpdate(Box::new(sample_environment())),
        "C++ environment golden != Rust sample environment"
    );
    let snap = std::fs::read(dir.join("golden_snapshot.bin")).unwrap();
    assert_eq!(
        decode_message(&snap).unwrap(),
        MessageT::Snapshot(Box::new(sample_snapshot())),
        "C++ snapshot golden != Rust sample snapshot (environment/outside)"
    );

    // Committed goldens carry the same pair.
    let committed = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let env = std::fs::read(committed.join("golden_environment.bin")).unwrap();
    assert_eq!(
        decode_message(&env).unwrap(),
        MessageT::EnvironmentUpdate(Box::new(sample_environment())),
        "committed environment golden != Rust sample environment"
    );
}

// ------------------------------------------------------ 10.1a (old reader)
#[test]
fn t10_1a_old_reader_ignores_environment() {
    use proto::generated_v1::cdda::ipc as v1;

    // (a) A standalone EnvironmentUpdate is an unknown union member to
    // the v1 decoder: it must unpack to NONE (ignored), never panic,
    // never error at the verifier level.
    let bytes = encode_message(&MessageT::EnvironmentUpdate(Box::new(
        sample_environment(),
    )));
    let env = v1::root_as_envelope(&bytes).expect("v1 verifier accepts the buffer");
    let unpacked = env.unpack();
    assert!(
        matches!(unpacked.message, v1::MessageT::NONE),
        "v1 reader must ignore the unknown EnvironmentUpdate member, got {:?}",
        unpacked.message
    );

    // (b) A Snapshot carrying the piggyback environment + outside
    // arrays decodes under v1 with every field it knows intact; the
    // new fields are simply absent from its view.
    let bytes = encode_message(&MessageT::Snapshot(Box::new(sample_snapshot())));
    let env = v1::root_as_envelope(&bytes).expect("v1 verifier accepts the buffer");
    match env.unpack().message {
        v1::MessageT::Snapshot(s) => {
            assert_eq!(s.seq, 1);
            assert_eq!(s.epoch, 1);
            assert_eq!(s.view_radius, 60);
            let blocks = s.blocks.expect("v1 snapshot blocks");
            assert_eq!(blocks.len(), 1);
            assert_eq!(blocks[0].terrain.as_ref().unwrap().len(), TILES);
            assert_eq!(
                blocks[0].terrain.as_ref().unwrap()[idx(5, 11)],
                TER_DOOR
            );
        }
        other => panic!("v1 reader expected Snapshot, got {other:?}"),
    }

    // (c) The new reader still decodes v1-produced data: a v1-encoded
    // snapshot (no environment, no outside) reads back with the new
    // fields absent, not defaulted-in.
    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let mut snap = v1::SnapshotT::default();
    snap.seq = 7;
    snap.epoch = 3;
    snap.view_radius = 12;
    let mut env_t = v1::EnvelopeT::default();
    env_t.message = v1::MessageT::Snapshot(Box::new(snap));
    let off = env_t.pack(&mut fbb);
    v1::finish_envelope_buffer(&mut fbb, off);
    let bytes = fbb.finished_data().to_vec();
    match decode_message(&bytes).unwrap() {
        MessageT::Snapshot(s) => {
            assert_eq!(s.seq, 7);
            assert!(s.environment.is_none(), "v1 data must read environment as absent");
        }
        other => panic!("new reader expected Snapshot, got {other:?}"),
    }
}
