//! Phase 5 rework Stage C gate: the four first recipes (grass,
//! dirt, wood floor, plaster) generated from the real UltiCa
//! families must meet the plan SS8 metric targets — seam ratio
//! < 1.15, mean deltaE_OK vs the family palette < 0.04, palette
//! <= 24 colors — and be byte-identical across two generations.
//!
//! The tileset is read from the MAIN tree checkout (fork-src is
//! gitignored and exists only there): from this crate's manifest
//! dir, `../../../cdda-voxel/fork-src/...` resolves correctly both
//! from a sibling worktree and from the main tree itself.

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

const FAMILIES: [(&str, RecipeKind); 4] = [
    ("t_grass", RecipeKind::Grass),
    ("t_dirt", RecipeKind::Dirt),
    ("t_floor", RecipeKind::WoodFloor),
    ("t_wall", RecipeKind::Plaster),
];

#[test]
fn stage_c_recipes_meet_plan_metrics() {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    for (base, kind) in FAMILIES {
        let entry = ts
            .entries
            .get(base)
            .unwrap_or_else(|| panic!("tileset has {base}"));
        let family = family_sprites(&ts, entry);
        assert!(
            !family.is_empty(),
            "{base}: family pool must not be empty"
        );
        let gen = generate(kind, &family);
        assert_eq!(
            (gen.image.width(), gen.image.height()),
            (TEXTURE_SIZE, TEXTURE_SIZE),
            "{base}: generated texture must be 32x32"
        );
        let m = metrics::evaluate(&gen.image, &gen.palette);
        println!(
            "{} ({}): family {} sprites, shading factor {:.2}, seam {:.3}, mean dE {:.4}, max dE {:.4}, palette {}",
            base,
            kind.name(),
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
            kind.name(),
            m.seam_ratio
        );
        assert!(
            m.mean_delta_e < 0.04,
            "{base} ({}): mean deltaE {:.4} must be < 0.04",
            kind.name(),
            m.mean_delta_e
        );
        assert!(
            m.palette_size <= 24,
            "{base} ({}): palette {} must be <= 24",
            kind.name(),
            m.palette_size
        );
        // Determinism: a second generation from the same family is
        // byte-identical.
        let again = generate(kind, &family);
        assert_eq!(
            gen.image.as_raw(),
            again.image.as_raw(),
            "{base} ({}): two generations must be byte-identical",
            kind.name()
        );
        assert_eq!(gen.palette, again.palette);
    }
}
