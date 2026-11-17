//! Stage G.3 rework, Step 0: approved-model regression gate.
//!
//! The models Joey has signed off on must not move a single
//! voxel while the view-faithful rework lands around them. Each
//! approved model is pinned by an FNV-1a (64-bit) hash over its
//! size and every voxel (index order; `None` feeds one 0 byte, a
//! cell feeds [1, r, g, b, a]). The constants below were computed
//! on the PRE-REWORK tree (HEAD a318e05) with
//! examples/hash_probe.rs. If this gate ever fails, the rework
//! touched an approved model — stop; do not re-pin.

use image::RgbaImage;
use std::path::PathBuf;
use voxelizer::*;

fn ultica_dir() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        manifest.join("../../../cdda-voxel/fork-src/tilesets/gfx/UltimateCataclysm"),
        manifest.join("../../fork-src/tilesets/gfx/UltimateCataclysm"),
    ];
    for c in &candidates {
        if c.join("tile_info.json").exists() {
            return c.clone();
        }
    }
    panic!("UltiCa tileset not found; tried {candidates:?}");
}

fn load(sprite_id: &str) -> RgbaImage {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    let entry = ts
        .entries
        .get(sprite_id)
        .unwrap_or_else(|| panic!("tileset has {sprite_id}"));
    composite_entry(&ts, entry).expect("sprite composites")
}

fn fnv_model(m: &PropModel) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    let mut feed = |bs: &[u8]| {
        for &b in bs {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
    };
    for s in m.size {
        feed(&s.to_le_bytes());
    }
    for v in &m.voxels {
        match v {
            None => feed(&[0]),
            Some(c) => feed(&[1, c[0], c[1], c[2], c[3]]),
        }
    }
    h
}

fn solid_hash(id: &str) -> (u64, [u32; 3]) {
    let sprite = load(id);
    let seg = segment_for(id, &sprite).unwrap_or_else(|| panic!("{id} segments"));
    let m = solid_model_for(id, &sprite, &seg);
    (fnv_model(&m), m.size)
}

#[test]
fn approved_solid_models_are_pinned() {
    // (id, pre-rework size, pre-rework FNV-1a hash)
    let cases: [(&str, [u32; 3], u64); 5] = [
        ("f_counter", [32, 12, 32], 0x5f8ae4c1c989e335),
        ("f_bookcase", [32, 36, 14], 0x38f2cab4a45c56ed),
        ("f_fridge", [32, 39, 16], 0x610f325837044965),
        ("f_locker", [32, 36, 11], 0x13b0bb8433473346),
        ("f_table", [32, 11, 32], 0x1258d2b757eb88cb),
    ];
    for (id, size, hash) in cases {
        let (h, sz) = solid_hash(id);
        assert_eq!(sz, size, "{id} solid model size changed");
        assert_eq!(
            h, hash,
            "{id} solid model voxels changed (approved model — do not re-pin)"
        );
    }
}

#[test]
fn approved_shelf_unit_is_pinned() {
    let sprite = load("f_bookcase");
    let seg = segment_for("f_bookcase", &sprite).expect("bookcase segments");
    let arch = archetype_for("f_bookcase").expect("bookcase has its archetype");
    assert_eq!(arch, Archetype::ShelfUnit);
    let m = build(arch, &sprite, &seg);
    assert_eq!(m.size, [32, 36, 14], "ShelfUnit size changed");
    assert_eq!(
        fnv_model(&m),
        0x4d61655ef4cc5e59,
        "ShelfUnit voxels changed (approved model — do not re-pin)"
    );
}
