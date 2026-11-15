// Decodes a harness-produced Snapshot payload with the REAL Phase 1 Rust
// proto crate (crates/proto) — cross-language proof for Phase 3 output.
use std::process::exit;

fn main() {
    let path = std::env::args().nth(1).expect("usage: decode <snapshot.bin>");
    let bytes = std::fs::read(&path).expect("read snapshot payload");
    match proto::decode_message(&bytes) {
        Ok(proto::MessageT::Snapshot(s)) => {
            let blocks = s.blocks.as_deref().unwrap_or(&[]);
            let tiles: usize = blocks
                .iter()
                .map(|b| b.terrain.as_ref().map_or(0, |t| t.len()))
                .sum();
            println!(
                "RUST-DECODE PASS snapshot seq={} epoch={} blocks={} tiles={} entities={} vehicles={} memory_blocks={}",
                s.seq,
                s.epoch,
                blocks.len(),
                tiles,
                s.entities.as_ref().map_or(0, |v| v.len()),
                s.vehicles.as_ref().map_or(0, |v| v.len()),
                s.memory_blocks.as_ref().map_or(0, |v| v.len())
            );
        }
        Ok(other) => {
            eprintln!("RUST-DECODE FAIL: unexpected message {other:?}");
            exit(1);
        }
        Err(e) => {
            eprintln!("RUST-DECODE FAIL: {e}");
            exit(1);
        }
    }
}
