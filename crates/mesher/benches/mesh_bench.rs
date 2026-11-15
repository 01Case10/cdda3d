//! Test 2.12 benchmark (plain timing binary, chosen over criterion to
//! keep the build light on small machines): mesh a 12x12 x N chunk where
//! N = cutaway depth in z-levels (4 levels here = 12 blocks tall), filled
//! with a house-like mix of blocks. Budget: under 2 ms per chunk
//! (single thread), per plan §11.

use std::time::Instant;

use mesher::block::block_for_tile;
use mesher::chunk::Chunk;
use mesher::mesher::{mesh_greedy, mesh_naive};
use mesher::scale::WorldScale;

fn main() {
    let scale = WorldScale::default();
    let levels = 4usize;
    let mut chunk = Chunk::new([0, 0], levels, scale);
    // Ground slab + a checker of walls/glass/foliage above it: a
    // non-trivial mask for the greedy mesher, not a single solid cube.
    for y in 0..12 {
        for x in 0..12 {
            chunk.set_tile(x, y, 0, 6, scale); // ground
            if (x + y) % 3 == 0 {
                chunk.set_tile(x, y, 1, 2, scale); // wall
            } else if (x + y) % 3 == 1 {
                chunk.set_tile(x, y, 1, 4, scale); // glass
            } else {
                chunk.set_tile(x, y, 1, 9, scale); // foliage
            }
            if x % 4 == 0 && y % 4 == 0 {
                chunk.set_tile(x, y, 2, 5, scale); // roof
            }
        }
    }
    let _ = block_for_tile(0);

    for (name, greedy) in [("greedy", true), ("naive", false)] {
        // Warm up, then time.
        for _ in 0..20 {
            std::hint::black_box(if greedy {
                mesh_greedy(&chunk.grid, chunk.origin(), chunk.grid.size, scale)
            } else {
                mesh_naive(&chunk.grid, chunk.origin(), chunk.grid.size, scale)
            });
        }
        let iters = 200;
        let start = Instant::now();
        let mut quads = 0;
        for _ in 0..iters {
            let m = if greedy {
                mesh_greedy(&chunk.grid, chunk.origin(), chunk.grid.size, scale)
            } else {
                mesh_naive(&chunk.grid, chunk.origin(), chunk.grid.size, scale)
            };
            quads = m.total_quads();
            std::hint::black_box(m);
        }
        let per_chunk_ms = start.elapsed().as_secs_f64() * 1000.0 / iters as f64;
        let verdict = if per_chunk_ms < 2.0 { "PASS (< 2 ms budget)" } else { "OVER BUDGET" };
        println!(
            "2.12 {name}: {per_chunk_ms:.4} ms/chunk (12x12x{levels} levels, {quads} quads) — {verdict}"
        );
    }
}
