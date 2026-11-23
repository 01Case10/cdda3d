//! Puddle gate (terrain item 2, 2026-11-23): a
//! puddle is its bg ground (grass, snow in winter)
//! with the FG's alpha footprint flattened into ONE
//! level water surface. The old route sent puddles
//! through `terrain_ground` on the FG sprite alone,
//! which fills the sprite's transparent rim with the
//! modal mud colour -- the whole 32x32 top came out
//! mud. The v1 blob raised the footprint as a dome
//! and read as a dirt mound (Joey's verdict), so the
//! shape is flat and level, the colours stay the
//! sprite's own, and the water read is carried by the
//! engine's water shading, recorded in the models
//! table as `surface: "water"` (this gate pins the
//! builder half of that contract: kind Ground,
//! surface Water, one shared surface level).
//!
//! Sprite facts (measured 2026-11-23): the t_puddle
//! FG is a 32x32 mud-brown irregular oval, 892/1024
//! px opaque (87.1%), all four corners transparent;
//! the tileset entry carries its underlay as bg
//! (t_grass / season grass / snow_shallow_center).

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

const PUDDLES: [&str; 4] = [
    "t_puddle",
    "t_puddle_season_summer",
    "t_puddle_season_autumn",
    "t_puddle_season_winter",
];

/// Topmost filled cell (y, colour) of one column.
fn column_top(m: &PropModel, x: u32, z: u32) -> Option<(u32, image::Rgba<u8>)> {
    (0..m.size[1])
        .rev()
        .find_map(|y| m.get(x, y, z).map(|c| (y, c)))
}

#[test]
fn puddle_is_flat_water_surface_on_its_bg_ground() {
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    for id in PUDDLES {
        let entry = ts.entries.get(id).unwrap_or_else(|| panic!("{id}: entry"));
        let fg = terrain_fg_image(&ts, entry).unwrap_or_else(|| panic!("{id}: fg sprite"));
        let bg = entry
            .bg
            .as_ref()
            .and_then(|r| load_sprite_image(&ts, entry, r))
            .unwrap_or_else(|| panic!("{id}: bg underlay must load"));
        let (m, kind, surface) =
            props::terrain_model_for(id, &fg, Some(&bg)).unwrap_or_else(|| panic!("{id}: model"));
        assert_eq!(kind, props::TerrainModelKind::Ground, "{id}: kind");
        assert_eq!(
            surface,
            Some(props::TerrainSurface::Water),
            "{id}: surface must be Water (the engine shades it as water)"
        );

        // Reference: the bg ground alone, built the
        // same way the emission builds a grass tile.
        let (base, _, _) = props::terrain_model_for("t_grass", &bg, None)
            .unwrap_or_else(|| panic!("{id}: bg ground model"));

        let (sw, sh) = (fg.width(), fg.height());
        let mut alpha_cols = 0usize;
        let mut level: Option<u32> = None;
        for z in 0..32u32 {
            for x in 0..32u32 {
                let fp = fg.get_pixel(x * sw / 32, z * sh / 32);
                let bp = bg.get_pixel(x * bg.width() / 32, z * bg.height() / 32);
                let (top, colour) =
                    column_top(&m, x, z).unwrap_or_else(|| panic!("{id}: empty column {x},{z}"));
                let (base_top, _) = column_top(&base, x, z).expect("base column");
                if fp[3] >= 128 {
                    alpha_cols += 1;
                    // Every footprint column tops out at
                    // the SAME level -- a flat water
                    // surface, not a dome and not the
                    // ground's relief.
                    match level {
                        None => level = Some(top),
                        Some(l) => assert_eq!(
                            top, l,
                            "{id}: footprint column {x},{z} top {top} breaks the \
                             flat surface level {l}"
                        ),
                    }
                    // The surface colour is the FG's own
                    // pixel -- sprite colours verbatim,
                    // no tint (the shader does the water
                    // work, not the palette).
                    assert_eq!(
                        colour, *fp,
                        "{id}: footprint column {x},{z} surface is not the fg colour"
                    );
                    // The level is the footprint's highest
                    // ground top: the surface is never
                    // buried by the ground's own relief.
                    assert!(
                        top >= base_top,
                        "{id}: footprint column {x},{z} surface {top} below its ground {base_top}"
                    );
                } else {
                    // Outside the footprint the tile is
                    // untouched bg ground: same height,
                    // the bg sprite's own colour -- never
                    // the modal mud fill.
                    assert_eq!(
                        top, base_top,
                        "{id}: outside-footprint column {x},{z} height changed"
                    );
                    assert_eq!(
                        colour, *bp,
                        "{id}: outside-footprint column {x},{z} is not bg ground colour"
                    );
                }
            }
        }
        // The footprint is the sprite's alpha fraction
        // (~87%): the surface covers exactly the
        // footprint, no more, no less.
        let alpha_frac = alpha_cols as f32 / 1024.0;
        assert!(
            (0.80..=0.93).contains(&alpha_frac),
            "{id}: sprite alpha fraction drifted: {alpha_frac}"
        );
        // The four corners are bg ground, not mud --
        // the exact failure of the old full slab.
        for (x, z) in [(0, 0), (31, 0), (0, 31), (31, 31)] {
            let (_, colour) = column_top(&m, x, z).expect("corner column");
            let bp = bg.get_pixel(x * bg.width() / 32, z * bg.height() / 32);
            assert_eq!(colour, *bp, "{id}: corner {x},{z} is not bg ground");
        }
    }
}

#[test]
fn puddle_without_underlay_still_models() {
    // Callers that only have the FG (debug tools) must
    // not lose the model entirely; it falls back to
    // the plain ground route and carries no surface.
    let ts = tileset::load_decomposed(&ultica_dir()).expect("UltiCa loads");
    let entry = ts.entries.get("t_puddle").expect("entry");
    let fg = terrain_fg_image(&ts, entry).expect("fg");
    let (m, kind, surface) = props::terrain_model_for("t_puddle", &fg, None).expect("model");
    assert_eq!(kind, props::TerrainModelKind::Ground);
    assert_eq!(surface, None);
    assert!(m.voxels.iter().any(|v| v.is_some()));
}
