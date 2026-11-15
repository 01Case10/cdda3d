//! Regenerate the Phase 7 visual-scene PNGs (gate 7.8) into
//! `crates/lighting/output/` without running the full test suite:
//! `cargo run -p lighting --bin render_scenes`.

use std::path::PathBuf;

use lighting::atmosphere::WeatherKind;
use lighting::fixtures;
use lighting::render::{render_isometric, render_top_down};
use lighting::{bake_chunk_map, Environment};

fn main() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("output");
    std::fs::create_dir_all(&dir).expect("create output dir");
    let night = Environment::new(0.0, WeatherKind::Clear);

    let map = fixtures::fake_house_map();
    let grid = fixtures::fake_house_night_light();
    render_top_down(&map, &grid, 0, night, 16)
        .write_png(&dir.join("house_night_topdown.png"))
        .expect("write house topdown");
    let baked = lighting::filter_levels(&bake_chunk_map(&map, &grid, night), &[0]);
    render_isometric(&baked, 8)
        .write_png(&dir.join("house_night_iso.png"))
        .expect("write house iso");

    let (fmap, fgrid) = fixtures::flashlight_scene();
    render_top_down(&fmap, &fgrid, 0, night, 12)
        .write_png(&dir.join("flashlight_field.png"))
        .expect("write flashlight");

    let (smap, sgrid) = fixtures::remembered_street_scene();
    render_top_down(&smap, &sgrid, 0, Environment::neutral(), 12)
        .write_png(&dir.join("remembered_street.png"))
        .expect("write street");

    println!("wrote Phase 7 scene PNGs to {}", dir.display());
}
