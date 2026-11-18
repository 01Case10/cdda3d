use std::path::PathBuf;
use voxelizer::*;

fn ultica_dir() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.join("../../fork-src/tilesets/gfx/UltimateCataclysm")
}

fn main() {
    let id = std::env::args().nth(1).expect("id");
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    let entry = ts.entries.get(&id).expect("entry");
    let sprite = terrain_fg_image(&ts, entry).expect("fg");
    println!("sprite {}x{}", sprite.width(), sprite.height());
    let (m, _kind) = props::terrain_model_for(&id, &sprite).expect("model");
    println!("model size {:?}", m.size);
    for y in (0..m.size[1]).step_by(6) {
        let mut n = 0;
        let (mut x0, mut x1) = (i32::MAX, -1);
        let (mut z0, mut z1) = (i32::MAX, -1);
        for z in 0..m.size[2] {
            for x in 0..m.size[0] {
                if m.get(x, y, z).is_some() {
                    n += 1;
                    x0 = x0.min(x as i32);
                    x1 = x1.max(x as i32);
                    z0 = z0.min(z as i32);
                    z1 = z1.max(z as i32);
                }
            }
        }
        println!("layer {y}: cells {n}, x span {x0}..{x1}, z span {z0}..{z1}");
    }
}
