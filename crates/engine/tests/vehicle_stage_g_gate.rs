//! Stage-G S1 gate (docs/vehicle-stage-g.md §5): the Rust
//! vehicle composer reproduces the Python prototype's car
//! geometry exactly. Per tile, the occupied-cell count and
//! an FNV-1a hash of the sorted occupied voxel indices must
//! equal the fixture generated from the prototype's exported
//! models (`fixtures/vehicle_stage_g_car.txt`) — the
//! prototype is the spec, so any constructor drift fails
//! here before it can reach a capture.
//!
//! Inputs are the prototype's own: the `car` def (copy-from
//! resolved) and ROOF flags from `fork-src` JSON, sprites
//! from the voxelizer's gallery asset set (built in-process
//! on first use, like the vehicle gallery example).

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use voxelizer::vehicle::{compose_vehicle, PartSpot, VehicleLayout};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn ensure_assets(root: &Path) -> PathBuf {
    let assets = root.join("target/gallery_assets");
    if !assets.join("materials.json").exists() {
        let tileset_dir = root.join("fork-src/cdda/gfx/UltimateCataclysm");
        let ts = tileset::load_decomposed(&tileset_dir).expect("tileset loads");
        let cfg = voxelizer::VoxelizerConfig {
            emit_v2: true,
            ..Default::default()
        };
        voxelizer::run(&ts, &cfg, &assets).expect("voxelizer run");
    }
    assets
}

fn read_json_dir(dir: &Path) -> Vec<serde_json::Value> {
    let mut out = Vec::new();
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();
    for f in files {
        let text = std::fs::read_to_string(&f).unwrap();
        if let Ok(serde_json::Value::Array(items)) =
            serde_json::from_str::<serde_json::Value>(&text)
        {
            out.extend(items);
        }
    }
    out
}

/// The prototype's def resolution: copy-from chains flattened,
/// location entries in definition order.
fn car_layout(root: &Path) -> VehicleLayout {
    let json = root.join("fork-src/cdda/data/json");
    let mut defs: HashMap<String, serde_json::Value> = HashMap::new();
    for v in read_json_dir(&json.join("vehicles")) {
        if let Some(k) = v.get("id").or_else(|| v.get("abstract")).and_then(|k| k.as_str()) {
            defs.insert(k.to_string(), v);
        }
    }
    fn resolve(
        defs: &HashMap<String, serde_json::Value>,
        vid: &str,
        out: &mut Vec<serde_json::Value>,
    ) {
        let v = &defs[vid];
        if let Some(parent) = v.get("copy-from").and_then(|c| c.as_str()) {
            resolve(defs, parent, out);
        }
        if let Some(parts) = v.get("parts").and_then(|p| p.as_array()) {
            out.extend(parts.iter().cloned());
        }
    }
    let mut entries = Vec::new();
    resolve(&defs, "car", &mut entries);

    let mut roof_ids: HashSet<String> = HashSet::new();
    for v in read_json_dir(&json.join("vehicleparts")) {
        if v.get("type").and_then(|t| t.as_str()) == Some("vehicle_part") {
            let flags = v.get("flags").and_then(|f| f.as_array());
            if flags.is_some_and(|f| f.iter().any(|x| x.as_str() == Some("ROOF"))) {
                if let Some(id) = v.get("id").and_then(|i| i.as_str()) {
                    roof_ids.insert(id.to_string());
                }
            }
        }
    }

    let mut layout = VehicleLayout::default();
    for e in &entries {
        let (x, y) = (
            e.get("x").and_then(|v| v.as_i64()).unwrap() as i32,
            e.get("y").and_then(|v| v.as_i64()).unwrap() as i32,
        );
        let spots = layout.tiles.entry((x, y, 0)).or_default();
        for p in e.get("parts").and_then(|p| p.as_array()).unwrap() {
            let name = match p {
                serde_json::Value::String(s) => s.clone(),
                other => other.get("part").and_then(|s| s.as_str()).unwrap().to_string(),
            };
            let base = name.split('#').next().unwrap().to_string();
            spots.push(PartSpot {
                roof: roof_ids.contains(&base),
                base,
                open: false,
                broken: false,
            });
        }
    }
    layout
}

fn sprite_resolver(assets: &Path) -> impl Fn(&str) -> Option<image::RgbaImage> {
    let text = std::fs::read_to_string(assets.join("materials.json")).unwrap();
    let doc: serde_json::Value = serde_json::from_str(&text).unwrap();
    let items = doc
        .get("materials")
        .cloned()
        .unwrap_or(serde_json::Value::Array(vec![]));
    let mut top_layer: HashMap<String, u32> = HashMap::new();
    if let Some(arr) = items.as_array() {
        for m in arr {
            if let (Some(id), Some(layer)) = (
                m.get("id").and_then(|i| i.as_str()),
                m.get("top_layer").and_then(|l| l.as_u64()),
            ) {
                top_layer.insert(id.to_string(), layer as u32);
            }
        }
    }
    let blocks = assets.join("blocks");
    let cache: RefCell<HashMap<String, Option<image::RgbaImage>>> = RefCell::new(HashMap::new());
    move |id: &str| {
        if let Some(hit) = cache.borrow().get(id) {
            return hit.clone();
        }
        let img = top_layer.get(id).and_then(|layer| {
            image::open(blocks.join(format!("layer_{layer:04}.png")))
                .ok()
                .map(|i| i.to_rgba8())
        });
        cache.borrow_mut().insert(id.to_string(), img.clone());
        img
    }
}

fn fnv1a(indices: &[u32]) -> u64 {
    let mut h: u64 = 14695981039346656037;
    for v in indices {
        for b in v.to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(1099511628211);
        }
    }
    h
}

#[test]
fn car_occupancy_matches_the_prototype() {
    let root = root();
    let assets = ensure_assets(&root);
    let layout = car_layout(&root);
    let models = compose_vehicle(&layout, &sprite_resolver(&assets));
    let mut got: BTreeMap<(i32, i32), (usize, u64)> = BTreeMap::new();
    for ((tx, ty, zl), model) in &models {
        assert_eq!(*zl, 0, "the fixture car is single-level");
        let indices: Vec<u32> = model
            .voxels
            .iter()
            .enumerate()
            .filter(|(_, v)| v.is_some())
            .map(|(i, _)| i as u32)
            .collect();
        got.insert((*tx, *ty), (indices.len(), fnv1a(&indices)));
    }

    let fixture = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/vehicle_stage_g_car.txt"),
    )
    .unwrap();
    let mut want: BTreeMap<(i32, i32), (usize, u64)> = BTreeMap::new();
    for line in fixture.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split_whitespace().collect();
        want.insert(
            (f[0].parse().unwrap(), f[1].parse().unwrap()),
            (f[2].parse().unwrap(), u64::from_str_radix(f[3], 16).unwrap()),
        );
    }

    assert_eq!(
        got.keys().collect::<Vec<_>>(),
        want.keys().collect::<Vec<_>>(),
        "composed tile set differs from the prototype's"
    );
    let mut bad = Vec::new();
    for (tile, w) in &want {
        let g = &got[tile];
        if g != w {
            bad.push(format!("tile {tile:?}: got {} cells/{:016x}, want {} cells/{:016x}",
                             g.0, g.1, w.0, w.1));
        }
    }
    assert!(bad.is_empty(), "occupancy drift vs the prototype:\n{}", bad.join("\n"));
}
