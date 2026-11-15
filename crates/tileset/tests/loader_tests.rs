//! Tileset loader tests: tile_info geometry, folder->sheet mapping,
//! decomposed parsing of the real UltiCa tree shape (fixture form).

use std::collections::BTreeMap;
use tileset::*;

#[test]
fn tile_info_all_sheet_sizes() {
    // The real UltiCa tile_info.json tuples (recon §5).
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fork-src/tilesets/gfx/UltimateCataclysm/tile_info.json"),
    )
    .expect("real tile_info.json present in sparse checkout");
    let (bw, bh, sheets) = parse_tile_info(&text).unwrap();
    assert_eq!((bw, bh), (32, 32));
    let get = |f: &str| sheets.get(f).unwrap();
    assert_eq!(
        (
            get("small.png").sprite_width,
            get("small.png").sprite_offset_x
        ),
        (20, 6)
    );
    assert_eq!(
        (
            get("human_body.png").sprite_height,
            get("human_body.png").sprite_offset_y
        ),
        (36, -8)
    );
    assert_eq!(get("human_body_plus.png").sprite_offset_y, -16);
    assert_eq!(get("tall.png").sprite_offset_y, -32);
    assert_eq!(
        (
            get("centered.png").sprite_offset_x,
            get("centered.png").sprite_offset_y
        ),
        (-16, -16)
    );
    assert_eq!(get("large.png").sprite_offset_y, -32);
    assert_eq!(get("large_ridden.png").sprite_offset_y, -27);
    assert_eq!(
        (
            get("huge.png").sprite_width,
            get("huge.png").sprite_height,
            get("huge.png").sprite_offset_y
        ),
        (64, 96, -64)
    );
    assert_eq!(
        (
            get("giant.png").sprite_width,
            get("giant.png").sprite_offset_x
        ),
        (96, -32)
    );
    assert!(get("filler.png").filler);
    assert!(get("fallback.png").fallback);
    assert!(
        get("fallback.png").source.is_some(),
        "per-sheet source present"
    );
}

#[test]
fn folder_to_sheet_mapping() {
    assert_eq!(
        sheet_file_for_folder("pngs_normal_32x32").unwrap(),
        "normal.png"
    );
    assert_eq!(
        sheet_file_for_folder("pngs_human_body_plus_32x48").unwrap(),
        "human_body_plus.png"
    );
    assert_eq!(
        sheet_file_for_folder("pngs_fillergiant_96x96").unwrap(),
        "fillergiant.png"
    );
    assert_eq!(
        sheet_file_for_folder("pngs_small_20x20").unwrap(),
        "small.png"
    );
    assert_eq!(
        sheet_file_for_folder("pngs_large_ridden_64x64").unwrap(),
        "large_ridden.png"
    );
}

#[test]
fn resolution_jump_limit() {
    // A looks_like cycle must terminate at the jump limit, not hang.
    let mut ts = Tileset::default();
    ts.base_width = 32;
    ts.base_height = 32;
    let mut ll = BTreeMap::new();
    ll.insert("a".to_string(), "b".to_string());
    ll.insert("b".to_string(), "a".to_string());
    let r = resolve(&ts, "a", &ll, None, "spring");
    assert_eq!(r.stage, Stage::Missing);
    assert!(r.jumps <= tileset::resolve::LOOKS_LIKE_JUMP_LIMIT);
}
