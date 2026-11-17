//! Phase 5 rework Stage F part 2 gate — generated wall end-caps.
//!
//! A wall sprite is a face-on illustration of the wall's side and
//! contains NO top-face pixels (docs/sprite-anatomy.md §4), so a
//! wall material's top layer must be generated from the family's
//! own palette (generate_wall_cap), never the sprite lying flat.
//!
//! Gates, per cited wall class (plaster / concrete / brick / wood):
//!   * cap != the old sprite-as-top output for the same family
//!   * cap meets the Stage C §8 targets against its (de-shaded)
//!     sprite-family palette: seam ratio < 1.15, mean ΔE_OK < 0.04,
//!     palette <= 24 — targets NOT relaxed
//!   * byte-identical on re-generation
//!   * caps are pairwise distinct across classes
//! And at the pipeline level (fixture tileset):
//!   * every non-wall material's top/side/bottom layers are
//!     byte-identical to the committed pre-F2 goldens
//!   * the fixture wall's side/bottom still match the goldens and
//!     only its top layer changed; a second run reproduces it
//!     byte-identically
//!
//! The real-UltiCa half reads the tileset from the MAIN tree
//! checkout (fork-src is gitignored and exists only there), the
//! same convention as stage_c_gate.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
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

const WALLS: [&str; 4] = ["t_wall", "t_concrete_wall", "t_brick_wall", "t_wall_wood"];

fn class_of(ts: &tileset::Tileset, entry: &tileset::TileEntry, id: &str) -> MaterialClass {
    let sprite = composite_entry(ts, entry).expect("wall sprite composites");
    let top = make_top_face(&sprite, ts, entry, 32);
    let palette = analyze_image(&top, 5).palette;
    classify(
        id,
        Some(TileKind::from_id_prefix(id)),
        Some(&palette),
        &Overrides::default(),
    )
    .class
}

#[test]
fn wall_caps_are_generated_palette_true_and_tileable() {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    let mut caps: Vec<(&str, Vec<u8>)> = Vec::new();
    for base in WALLS {
        let entry = ts
            .entries
            .get(base)
            .unwrap_or_else(|| panic!("tileset has {base}"));
        let family = family_sprites(&ts, entry);
        assert!(!family.is_empty(), "{base}: family pool must not be empty");
        let class = class_of(&ts, entry, base);
        let gen = generate_wall_cap(class, &family);
        assert_eq!(
            (gen.image.width(), gen.image.height()),
            (TEXTURE_SIZE, TEXTURE_SIZE),
            "{base}: cap must be 32x32"
        );
        assert!(
            gen.image.pixels().all(|p| p[3] == 255),
            "{base}: cap must be fully opaque"
        );

        // The old output: the sprite itself as the top face.
        let sprite = composite_entry(&ts, entry).unwrap();
        let old_top = make_top_face(&sprite, &ts, entry, 32);
        assert_ne!(
            gen.image.as_raw(),
            old_top.as_raw(),
            "{base} ({}): cap must be generated, not the sprite-as-top",
            class.name()
        );

        let m = metrics::evaluate(&gen.image, &gen.palette);
        println!(
            "{} ({}): family {} sprites, shading factor {:.2}, seam {:.3}, mean dE {:.4}, max dE {:.4}, palette {}",
            base,
            class.name(),
            family.len(),
            gen.analysis.shading.shading_factor,
            m.seam_ratio,
            m.mean_delta_e,
            m.max_delta_e,
            m.palette_size
        );
        assert!(
            m.seam_ratio < 1.15,
            "{base} ({}): seam ratio {:.3} must be < 1.15",
            class.name(),
            m.seam_ratio
        );
        assert!(
            m.mean_delta_e < 0.04,
            "{base} ({}): mean deltaE {:.4} must be < 0.04",
            class.name(),
            m.mean_delta_e
        );
        assert!(
            m.palette_size <= 24,
            "{base} ({}): palette {} must be <= 24",
            class.name(),
            m.palette_size
        );

        let again = generate_wall_cap(class, &family);
        assert_eq!(
            gen.image.as_raw(),
            again.image.as_raw(),
            "{base} ({}): two generations must be byte-identical",
            class.name()
        );
        assert_eq!(gen.palette, again.palette);
        caps.push((base, gen.image.as_raw().clone()));
    }
    // Distinguishable per wall class: no two caps are the same image.
    for i in 0..caps.len() {
        for j in i + 1..caps.len() {
            assert_ne!(
                caps[i].1, caps[j].1,
                "caps for {} and {} must differ",
                caps[i].0, caps[j].0
            );
        }
    }
}

// --- Pipeline level: fixture non-wall byte-identity ---------------

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixture")
}

fn fixture_cfg() -> VoxelizerConfig {
    let mut looks = BTreeMap::new();
    looks.insert("t_dirt_lookalike".to_string(), "t_dirt".to_string());
    VoxelizerConfig {
        season: "spring".into(),
        tile_size: 32,
        looks_like: looks,
        extra_ids: vec![
            "t_dirt_lookalike".into(),
            "t_rock".into(),
            "t_seasonal".into(),
            "t_totally_unknown".into(),
        ],
        tileset_license_text: std::fs::read_to_string(fixture_dir().join("LICENSE.txt")).ok(),
        ..Default::default()
    }
}

fn tmpdir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("vox_f2caps_{}_{}", std::process::id(), tag));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn fixture_only_wall_top_changes() {
    let golden_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let golden_mats: Vec<MaterialOut> =
        serde_json::from_str(&std::fs::read_to_string(golden_root.join("materials.json")).unwrap())
            .unwrap();
    let layer_bytes = |root: &Path, layer: usize| {
        std::fs::read(root.join("blocks").join(format!("layer_{layer:04}.png"))).unwrap()
    };

    let ts = tileset::load_decomposed(&fixture_dir()).expect("fixture loads");
    let out = tmpdir("run1");
    run(&ts, &fixture_cfg(), &out).expect("voxelizer run");
    let mats: Vec<MaterialOut> =
        serde_json::from_str(&std::fs::read_to_string(out.join("materials.json")).unwrap())
            .unwrap();
    assert_eq!(mats.len(), golden_mats.len(), "material set must not change");

    for (got, want) in mats.iter().zip(golden_mats.iter()) {
        assert_eq!(got.id, want.id);
        assert_eq!(
            (got.top_layer, got.side_layer, got.bottom_layer),
            (want.top_layer, want.side_layer, want.bottom_layer),
            "{}: layer indices must not move",
            got.id
        );
        let wall = got.id == "t_wall";
        for (name, layer) in [
            ("top", got.top_layer),
            ("side", got.side_layer),
            ("bottom", got.bottom_layer),
        ] {
            if wall && name == "top" {
                continue; // the one deliberate change; checked below
            }
            let g = layer_bytes(&out, layer);
            let w = layer_bytes(&golden_root, layer);
            assert_eq!(
                g, w,
                "{}: {} layer must stay byte-identical to the golden",
                got.id, name
            );
        }
    }
    // The wall top is a generated cap, not the sprite-as-top the
    // pipeline used to emit.
    let wall_entry = ts.entries.get("t_wall").unwrap().clone();
    let comp = composite_entry(&ts, &wall_entry).unwrap();
    let sprite_face = make_top_face(&comp, &ts, &wall_entry, 32);
    let wall_mat = mats.iter().find(|m| m.id == "t_wall").unwrap();
    let cap = image::open(out.join("blocks").join(format!("layer_{:04}.png", wall_mat.top_layer)))
        .unwrap()
        .to_rgba8();
    assert_ne!(
        cap.as_raw(),
        sprite_face.as_raw(),
        "t_wall: top layer must be the generated end-cap, not the sprite"
    );

    // Run-level determinism: a second fixture run reproduces the
    // wall cap byte-identically.
    let out2 = tmpdir("run2");
    run(&ts, &fixture_cfg(), &out2).expect("voxelizer run 2");
    let wall = mats.iter().find(|m| m.id == "t_wall").unwrap();
    assert_eq!(
        layer_bytes(&out, wall.top_layer),
        layer_bytes(&out2, wall.top_layer),
        "t_wall cap must be byte-identical across runs"
    );
}
