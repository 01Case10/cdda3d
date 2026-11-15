//! Phase 2d shared-table gate: `mesher::connect` is the canonical
//! mask -> subtile mapping every consumer (voxelizer data, RT
//! selection, the future mesher path) resolves through. The table
//! itself is the game's rule, already pinned from the source side
//! by voxelizer gate 5.5; this test pins the shared copy so the two
//! can never drift (the voxelizer delegates here — see
//! `voxelizer::multitile`).

use mesher::connect::{all_mask_mappings, mask_from_connects, subtile_for_mask};

#[test]
fn mask_to_subtile_table_matches_the_game_rule() {
    let expected: [(u8, &str); 16] = [
        (0, "unconnected"),
        (1, "end_piece"),
        (2, "end_piece"),
        (4, "end_piece"),
        (8, "end_piece"),
        (3, "corner"),
        (6, "corner"),
        (9, "corner"),
        (12, "corner"),
        (5, "edge"),
        (10, "edge"),
        (7, "t_connection"),
        (11, "t_connection"),
        (13, "t_connection"),
        (14, "t_connection"),
        (15, "center"),
    ];
    for (mask, want) in expected {
        assert_eq!(subtile_for_mask(mask).0, want, "mask {mask:04b}");
    }
    // Rotations stay in range for the variant arrays
    // (corner/tee/end: 4, edge: 2, center/unconnected: 1).
    for m in 0..16u8 {
        let (sub, rot) = subtile_for_mask(m);
        let max = match sub {
            "edge" => 2,
            "center" | "unconnected" => 1,
            _ => 4,
        };
        assert!(rot < max, "mask {m} subtile {sub} rot {rot}");
    }
    // The bulk mapping agrees with the point function, in order.
    let all = all_mask_mappings();
    assert_eq!(all.len(), 16);
    for (i, (m, s, r)) in all.iter().enumerate() {
        assert_eq!(*m, i as u8);
        assert_eq!((*s, *r), subtile_for_mask(*m));
    }
    // Bit order: N=1, E=2, S=4, W=8.
    assert_eq!(mask_from_connects(true, false, false, false), 1);
    assert_eq!(mask_from_connects(false, true, false, false), 2);
    assert_eq!(mask_from_connects(false, false, true, false), 4);
    assert_eq!(mask_from_connects(false, false, false, true), 8);
    assert_eq!(mask_from_connects(true, false, true, false), 5);
    assert_eq!(subtile_for_mask(5).0, "edge", "N+S is a straight run");
}
