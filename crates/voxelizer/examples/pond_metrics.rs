//! Pond circularity metrics for Stage H tuning (worktree tool).
//! Water region = material_at classification over a mosaic of
//! shore/bank tiles, exactly as stage_e_gate composes it.
//! Prints: water area, best-fit disk IoU (center fixed at pond
//! center, radius searched), radial boundary mean/std/min/max,
//! corner-tile coverage (3x3), straight-edge first-water row.
use voxelizer::*;

fn water_grid(pond: i32, margin: i32) -> (Vec<bool>, usize) {
    // mosaic is (pond + 2*margin) tiles square; pond centered.
    let tiles = pond + 2 * margin;
    let size = (tiles * 32) as usize;
    let is_water = |tx: i32, ty: i32| {
        (margin..margin + pond).contains(&tx) && (margin..margin + pond).contains(&ty)
    };
    let mut grid = vec![false; size * size];
    for ty in 0..tiles {
        for tx in 0..tiles {
            let mask = mask_for(is_water, tx, ty);
            let cw = is_water(tx, ty);
            if !cw && mask == 0 {
                continue;
            }
            for y in 0..32usize {
                for x in 0..32usize {
                    if material_at(mask, cw, x as u32, y as u32) == ShoreMaterial::Water {
                        grid[(ty as usize * 32 + y) * size + (tx as usize * 32 + x)] = true;
                    }
                }
            }
        }
    }
    (grid, size)
}

fn report(pond: i32) {
    let (grid, size) = water_grid(pond, 1);
    let c = (size as f32 - 1.0) / 2.0;
    let area = grid.iter().filter(|&&w| w).count();
    // Best disk IoU.
    let mut best = (0.0f32, 0.0f32);
    let mut r = 4.0f32;
    while r < size as f32 {
        let (mut inter, mut union) = (0u32, 0u32);
        for y in 0..size {
            for x in 0..size {
                let d = ((x as f32 - c).powi(2) + (y as f32 - c).powi(2)).sqrt();
                let disk = d <= r;
                let w = grid[y * size + x];
                if disk && w {
                    inter += 1;
                }
                if disk || w {
                    union += 1;
                }
            }
        }
        let iou = inter as f32 / union as f32;
        if iou > best.0 {
            best = (iou, r);
        }
        r += 0.5;
    }
    // Radial boundary.
    let mut radii = Vec::new();
    for a in 0..360 {
        let th = (a as f32).to_radians();
        let (dx, dy) = (th.cos(), th.sin());
        let mut last = 0.0f32;
        let mut d = 0.0f32;
        while d < size as f32 {
            let x = (c + dx * d).round() as i32;
            let y = (c + dy * d).round() as i32;
            if x < 0 || y < 0 || x >= size as i32 || y >= size as i32 {
                break;
            }
            if grid[y as usize * size + x as usize] {
                last = d;
            } else if d > 4.0 {
                break;
            }
            d += 0.5;
        }
        radii.push(last);
    }
    let mean = radii.iter().sum::<f32>() / radii.len() as f32;
    let var = radii.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / radii.len() as f32;
    let min = radii.iter().cloned().fold(f32::MAX, f32::min);
    let max = radii.iter().cloned().fold(0.0f32, f32::max);
    println!(
        "pond {pond}x{pond}: area {area} px, best-disk IoU {:.3} @ r={:.1}, radial mean {:.1} std {:.2} min {:.1} max {:.1}",
        best.0,
        best.1,
        mean,
        var.sqrt(),
        min,
        max
    );
}

fn dump() {
    let (grid, size) = water_grid(3, 1);
    let c = (size as f32 - 1.0) / 2.0;
    let mut best = (0.0f32, 0, 0);
    for y in 0..size { for x in 0..size { if grid[y*size+x] {
        let d = ((x as f32-c).powi(2)+(y as f32-c).powi(2)).sqrt();
        if d > best.0 { best = (d, x, y); }
    }}}
    println!("farthest water px: {:?} center {c}", best);
    for y in (0..size).step_by(2) {
        let mut line = String::new();
        for x in (0..size).step_by(2) {
            line.push(if grid[y*size+x] { '#' } else { '.' });
        }
        println!("{line}");
    }
}

fn main() {
    dump();
    report(3);
    report(5);
    report(7);
    // Corner coverage for the 3x3 (gate metric).
    let is_water = |tx: i32, ty: i32| (1..=3).contains(&tx) && (1..=3).contains(&ty);
    let mut water_px = 0u32;
    for (tx, ty) in [(1, 1), (3, 1), (1, 3), (3, 3)] {
        let mask = mask_for(is_water, tx, ty);
        for y in 0..32 {
            for x in 0..32 {
                if material_at(mask, true, x, y) == ShoreMaterial::Water {
                    water_px += 1;
                }
            }
        }
    }
    println!(
        "3x3 corner-tile water coverage: {:.1}%",
        100.0 * water_px as f32 / (4 * 32 * 32) as f32
    );
    // Straight edge (gate metric).
    let straight_mask = 0b1111_1110u8;
    let first_water = (0..32)
        .find(|&y| material_at(straight_mask, true, 16, y) == ShoreMaterial::Water)
        .unwrap_or(99);
    println!("straight-edge first water at center column: y={first_water}");
    // Straight edge profile across the tile (is the waterline flat?)
    let profile: Vec<String> = (0..32)
        .map(|x| {
            let fw = (0..32)
                .find(|&y| material_at(straight_mask, true, x, y) == ShoreMaterial::Water)
                .map(|y| y.to_string())
                .unwrap_or_else(|| "-".into());
            fw
        })
        .collect();
    println!("straight-edge first-water profile x=0..31: {}", profile.join(","));
}
