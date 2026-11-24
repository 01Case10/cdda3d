//! Regression gate — partial memory blocks (Phase 11 follow-up,
//! found by S3).
//!
//! The real exporter's memory blocks are PARTIAL by construction:
//! `Exporter::build_memory_blocks` (fork/src/voxel_ipc/
//! voxel_exporter.cpp:164-167) groups memorized tiles into 12x12
//! blocks and explicitly leaves the block's never-memorized tiles
//! BLANK — "Tiles in a memory block that were never memorized stay
//! BLANK." The per-tile `lit` array is therefore authoritative:
//! MEMORIZED tiles were memorized; BLANK tiles were not.
//!
//! The proto adapter used to stamp MEMORIZED on EVERY tile of a
//! memory block, ignoring the array, so never-seen tiles inside a
//! partial block surfaced in the grid as Memorized (desaturated +
//! dimmed, and meshed) instead of Hidden (black, not meshed, not
//! in the RT tree). Fixtures never exposed it — the stub house has
//! no memory blocks at all; the sim's three_rooms snapshot does
//! produce the partial shape (engine sim_visibility_gate pins the
//! wire half of this contract).
//!
//! Correct per-tile behaviour, grounded in the exporter above and
//! in `lighting::mapping::visibility_of` (BLANK -> Hidden,
//! MEMORIZED -> Memorized, everything unexpected fails closed):
//! within a memory block, a tile is Memorized iff its wire lit is
//! MEMORIZED; any other value means "not memorized" and reads
//! BLANK/Hidden. A memory block must never make a tile Visible —
//! the memory layer is remembered-but-not-currently-visible by
//! definition (schema comment on `Snapshot.memory_blocks`).

use lighting::mapping::{is_meshed, Visibility};
use lighting::proto_adapter::grid_from_snapshot;
use proto::{LitLevel, SnapshotT, SubmapBlockT, Vec3iT};

/// A memory block in the exporter's own shape: terrain/furniture
/// only, `seen` all zero, `lit` supplied by the caller.
fn memory_block(lit: Vec<LitLevel>) -> SubmapBlockT {
    let mut b = SubmapBlockT::default();
    b.origin = Some(Vec3iT { x: 0, y: 0, z: 0 });
    b.terrain = Some(vec![1; 144]);
    b.furniture = Some(vec![0; 144]);
    b.seen = Some(vec![0; 144]);
    b.lit = Some(lit);
    b
}

fn snapshot_with_memory(mem: SubmapBlockT) -> SnapshotT {
    let mut snap = SnapshotT::default();
    snap.memory_blocks = Some(vec![mem]);
    snap
}

#[test]
fn partial_memory_block_blank_tiles_read_hidden() {
    // West half of the block memorized, east half never memorized
    // (BLANK on the wire) — the exporter's partial-block shape.
    let mut lit = vec![LitLevel::BLANK; 144];
    for ly in 0..12 {
        for lx in 0..6 {
            lit[ly * 12 + lx] = LitLevel::MEMORIZED;
        }
    }
    let g = grid_from_snapshot(&snapshot_with_memory(memory_block(lit)));

    // Memorized half: Memorized, and meshed (dimmed memory geometry).
    let m = g.get(3, 6, 0);
    assert_eq!(m.lit, LitLevel::MEMORIZED);
    assert_eq!(m.visibility(), Visibility::Memorized);
    assert!(is_meshed(m.visibility()));

    // Never-memorized half: Hidden — not Memorized. This is the
    // S3 divergence: pre-fix these tiles read MEMORIZED.
    let h = g.get(9, 6, 0);
    assert_eq!(
        h.lit,
        LitLevel::BLANK,
        "never-memorized tile in a memory block must stay BLANK"
    );
    assert_eq!(h.visibility(), Visibility::Hidden);
    assert!(!is_meshed(h.visibility()));

    assert_eq!(g.count_visibility(Visibility::Memorized), 72);
    assert_eq!(g.count_visibility(Visibility::Hidden), 72);
    assert_eq!(g.count_visibility(Visibility::Visible), 0);
}

#[test]
fn memory_block_never_makes_a_tile_visible() {
    // Adapter-side enforcement, corrected form: a sender that wrote
    // visible levels into a memory block must not get Visible (or
    // Memorized) tiles out of it — a non-MEMORIZED entry in the
    // memory layer is a never-memorized tile and fails closed to
    // BLANK/Hidden (mapping::visibility_of's fail-closed rule).
    let mut lit = vec![LitLevel::LIT; 144];
    lit[0] = LitLevel::BRIGHT;
    lit[1] = LitLevel::MEMORIZED;
    let g = grid_from_snapshot(&snapshot_with_memory(memory_block(lit)));

    let written_visible = g.get(5, 5, 0);
    assert_eq!(written_visible.lit, LitLevel::BLANK);
    assert_eq!(written_visible.visibility(), Visibility::Hidden);
    assert_eq!(g.get(0, 0, 0).visibility(), Visibility::Hidden);

    // The one genuinely memorized tile still reads Memorized.
    let m = g.get(1, 0, 0);
    assert_eq!(m.lit, LitLevel::MEMORIZED);
    assert_eq!(m.visibility(), Visibility::Memorized);

    assert_eq!(g.count_visibility(Visibility::Visible), 0);
    assert_eq!(g.count_visibility(Visibility::Memorized), 1);
    assert_eq!(g.count_visibility(Visibility::Hidden), 143);
}
