//! Prop gallery (one-off showcase, 2026-11-17): run the
//! voxelizer over the real UltiCa tileset with v2 model
//! emission, then stamp every furniture prop model it
//! produced into the engine's sample scene on a grid and
//! capture the frame. Not a gate; a visual showcase for
//! the completed 351/351 prop coverage.

use std::path::PathBuf;

use engine::capture::{run_capture, CaptureArgs};
use engine::frame::PropScene;
use engine::materials::MaterialTable;
use engine::snapshot_world::FurniturePlacement;

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let tileset_dir = root.join("fork-src/tilesets/gfx/UltimateCataclysm");
    let assets = root.join("target/gallery_assets");
    let out_png = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target/gallery.png"));

    // 1. Voxelizer v2 run over the real tileset (models
    //    for every furniture material that has one).
    //    Skipped when a previous run's output is present.
    if assets.join("materials.json").exists() {
        println!("reusing existing voxelizer output in {}", assets.display());
    } else {
        let ts = tileset::load_decomposed(&tileset_dir).expect("tileset loads");
        let cfg = voxelizer::VoxelizerConfig {
            emit_v2: true,
            ..Default::default()
        };
        let report = voxelizer::run(&ts, &cfg, &assets).expect("voxelizer run");
        println!("voxelizer run: {report:?}");
    }

    // 2. Furniture ids that carry a model, via the v2
    //    materials.json (material_index -> id), joined
    //    through the engine loader.
    let mats: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(assets.join("materials.json")).unwrap())
            .unwrap();
    let table = MaterialTable::load(&assets).expect("material table loads");
    let mut ids: Vec<String> = Vec::new();
    if let Some(models) = mats["models"].as_array() {
        for e in models {
            let mi = e["material_index"].as_u64().unwrap() as usize;
            if let Some(m) = mats["materials"].as_array().unwrap().get(mi) {
                ids.push(m["id"].as_str().unwrap().to_string());
            }
        }
    }
    ids.sort();
    ids.dedup();
    println!("furniture models available: {}", ids.len());

    // Showcase pieces first, then the rest in id order.
    let priority = [
        "f_bookcase", "f_vending_c", "f_exercise", "f_pinball_machine",
        "f_wind_mill", "f_exodii_portal_tower", "f_machinery_light",
        "f_drill_press", "f_hydraulic_press", "f_alembic", "f_mannequin",
        "f_anvil", "f_orrery", "f_birdbath", "f_hammock", "f_sofa",
        "f_armchair", "f_table", "f_fridge", "f_oven", "f_bathtub",
        "f_bed", "f_wardrobe", "f_dresser", "f_exercise",
        "f_traffic_light", "f_small_satelitte_dish", "f_scrap_antenna",
        "f_alien_anemone", "f_chair", "f_deckchair", "f_winter_wreath",
    ];
    let mut ordered: Vec<String> = Vec::new();
    for p in priority {
        if ids.iter().any(|i| i == p) && !ordered.iter().any(|i| i == p) {
            ordered.push(p.to_string());
        }
    }
    for id in &ids {
        if !ordered.contains(id) {
            ordered.push(id.clone());
        }
    }

    // 3. Grid placements over the sample scene's grass,
    //    skipping the house footprint.
    let mut slots: Vec<(i32, i32)> = Vec::new();
    for gy in (1..24).step_by(2) {
        for gx in (1..24).step_by(2) {
            if (2..=11).contains(&gx) && (2..=10).contains(&gy) {
                continue; // the house
            }
            slots.push((gx, gy));
        }
    }
    let mut models: Vec<(u32, engine::materials::PropModelData)> = Vec::new();
    let mut placements: Vec<FurniturePlacement> = Vec::new();
    let mut skipped_oversize = 0usize;
    for id in &ordered {
        if placements.len() >= slots.len() {
            break;
        }
        let Some(mat) = table.get(id) else { continue };
        let Some(model) = &mat.model else { continue };
        // The fine stamper refuses models wider or
        // deeper than one tile (FINE = 32): those are
        // the 64px-sprite builds, which belong to
        // multitile furniture in the real game.
        if model.size[0] > 32 || model.size[2] > 32 {
            skipped_oversize += 1;
            continue;
        }
        let furn_id = 1000 + models.len() as u32;
        models.push((furn_id, model.clone()));
        let (x, y) = slots[placements.len()];
        placements.push(FurniturePlacement {
            x,
            y,
            level: 0,
            furn_id,
            quarter_turns: 0,
        });
    }
    println!(
        "placing {} props ({} oversize models skipped)",
        placements.len(),
        skipped_oversize
    );

    // Palette quantisation for the showcase: the frame
    // builder gives every distinct prop colour its own
    // texture-array layer, and this VM's software
    // adapter caps the array at 256 layers (a full
    // gallery has ~3,600 distinct colours). Remap every
    // placed model onto one shared 240-colour palette --
    // the most frequent colours stay exact, the rest go
    // to their nearest neighbour. Only this example's
    // copies are remapped; the emitted models are
    // untouched.
    {
        use std::collections::HashMap;
        let mut freq: HashMap<[u8; 3], u64> = HashMap::new();
        for (_, m) in &models {
            let mut per: HashMap<u16, u64> = HashMap::new();
            for v in &m.voxels {
                if *v != engine::materials::PropModelData::EMPTY {
                    *per.entry(*v).or_default() += 1;
                }
            }
            for (idx, n) in per {
                *freq.entry(m.palette[idx as usize]).or_default() += n;
            }
        }
        let mut ranked: Vec<([u8; 3], u64)> = freq.into_iter().collect();
        ranked.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
        let shared: Vec<[u8; 3]> = ranked.iter().take(160).map(|(c, _)| *c).collect();
        let shared_idx: HashMap<[u8; 3], u16> = shared
            .iter()
            .enumerate()
            .map(|(i, c)| (*c, i as u16))
            .collect();
        let mut cache: HashMap<[u8; 3], u16> = HashMap::new();
        let nearest = |c: [u8; 3], cache: &mut HashMap<[u8; 3], u16>| -> u16 {
            if let Some(i) = shared_idx.get(&c) {
                return *i;
            }
            if let Some(i) = cache.get(&c) {
                return *i;
            }
            let mut best = (0u16, u64::MAX);
            for (i, s) in shared.iter().enumerate() {
                let d: u64 = (0..3)
                    .map(|k| {
                        let dd = c[k] as i64 - s[k] as i64;
                        (dd * dd) as u64
                    })
                    .sum();
                if d < best.1 {
                    best = (i as u16, d);
                }
            }
            cache.insert(c, best.0);
            best.0
        };
        for (_, m) in &mut models {
            let remap: Vec<u16> = m
                .palette
                .iter()
                .map(|c| nearest(*c, &mut cache))
                .collect();
            for v in &mut m.voxels {
                if *v != engine::materials::PropModelData::EMPTY {
                    *v = remap[*v as usize];
                }
            }
            m.palette = shared.clone();
        }
        println!("quantised to a shared {}-colour palette", shared.len());
    }

    // 4. Capture the engine frame. The RT renderer is
    //    the default here: the raster path packs every
    //    distinct prop colour into a texture array, and
    //    this VM's software adapter caps that array at
    //    256 layers -- a full gallery has thousands.
    let use_rt = !std::env::args().any(|a| a == "--raster");
    let args = CaptureArgs {
        snapshot: "sample-scene".to_string(),
        textures: Some(assets.clone()),
        renderer: if use_rt {
            engine::capture::RendererKind::Rt
        } else {
            engine::capture::RendererKind::Raster
        },
        props: Some(PropScene { models, placements }),
        camera_target: (12.0, 12.0),
        yaw_deg: 45.0,
        pitch_deg: 38.0,
        distance: 30.0,
        size: (1600, 900),
        out: out_png.clone(),
        ..Default::default()
    };
    match run_capture(&args) {
        Ok(o) => println!(
            "wrote {} ({}x{}, {} quads, {} colors)",
            o.out.display(),
            o.size.0,
            o.size.1,
            o.quads,
            o.distinct_colors
        ),
        Err(e) => {
            eprintln!("capture failed: {e}");
            std::process::exit(1);
        }
    }
}
