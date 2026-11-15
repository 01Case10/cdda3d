//! Phase 5 texture hookup for the RT renderer: turn a voxelizer
//! output directory (`blocks/layer_NNNN.png` + `materials.json`,
//! loaded via `materials::MaterialTable`) into a treebuild
//! `TextureSpec` covering exactly the materials a scene uses.
//!
//! The full UltiCa output has 26,389 layers — far beyond any GPU
//! texture-array limit — so the spec is *scene-local*: only the
//! layers referenced by the requested materials are decoded and
//! uploaded, remapped to a compact array. The numeric material id
//! -> game id string resolution is the caller's (`id_map`): fake
//! data uses `rt::fake_material_ids`; real scenes will pass the
//! TileDefs mapping once Phase 4 wires snapshots in.

use std::collections::HashMap;
use std::path::Path;

use treebuild::render::TextureSpec;

use crate::materials::MaterialTable;

/// What a spec build resolved, for capture reporting and gates.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct TextureStats {
    /// Materials (of those requested) that got a texture entry.
    pub textured_materials: usize,
    /// Requested materials with no voxelizer entry (stay flat).
    pub missing_materials: usize,
    /// Layers uploaded into the scene texture array.
    pub layers_uploaded: usize,
}

/// Build a `TextureSpec` for the materials in `id_map`
/// (numeric material id -> game id string) from the voxelizer
/// output in `dir`. Materials that fail to resolve or whose layer
/// PNGs are missing/mis-sized are skipped (counted in the stats,
/// shaded flat) rather than failing the whole scene.
pub fn build_texture_spec(
    table: &MaterialTable,
    dir: &Path,
    id_map: &HashMap<u32, String>,
) -> Result<(TextureSpec, TextureStats), String> {
    let tile = table.tile_size;
    let mut stats = TextureStats::default();
    // Source layer index -> compact layer index, in first-use order.
    let mut compact_of: HashMap<usize, u32> = HashMap::new();
    let mut layers: Vec<Vec<u8>> = Vec::new();
    let mut load_layer = |src: usize,
                          compact_of: &mut HashMap<usize, u32>,
                          layers: &mut Vec<Vec<u8>>|
     -> Result<u32, String> {
        if let Some(&c) = compact_of.get(&src) {
            return Ok(c);
        }
        let path = dir.join("blocks").join(format!("layer_{src:04}.png"));
        let img = image::open(&path)
            .map_err(|e| format!("{}: {e}", path.display()))?
            .to_rgba8();
        if img.width() != tile || img.height() != tile {
            return Err(format!(
                "{}: {}x{} but tile_size is {tile}",
                path.display(),
                img.width(),
                img.height()
            ));
        }
        let idx = layers.len() as u32;
        layers.push(img.into_raw());
        compact_of.insert(src, idx);
        Ok(idx)
    };

    let max_id = id_map.keys().copied().max().unwrap_or(0);
    let mut material_layers = vec![[0u32; 4]; (max_id + 1) as usize];
    // Deterministic order (BTreeMap-style) so specs are reproducible.
    let mut ids: Vec<u32> = id_map.keys().copied().collect();
    ids.sort_unstable();
    for num_id in ids {
        let game_id = &id_map[&num_id];
        let Some(mat) = table.get(game_id) else {
            stats.missing_materials += 1;
            continue;
        };
        let resolved = (|| {
            let top = load_layer(mat.top_layer, &mut compact_of, &mut layers)?;
            let side = load_layer(mat.side_layer, &mut compact_of, &mut layers)?;
            let bottom = load_layer(mat.bottom_layer, &mut compact_of, &mut layers)?;
            Ok::<[u32; 4], String>([top, side, bottom, 1])
        })();
        match resolved {
            Ok(entry) => {
                material_layers[num_id as usize] = entry;
                stats.textured_materials += 1;
            }
            Err(e) => {
                eprintln!("voxel-engine: material {game_id} stays flat: {e}");
                stats.missing_materials += 1;
            }
        }
    }
    stats.layers_uploaded = layers.len();
    if layers.is_empty() {
        // TextureSpec requires at least one layer; give it the
        // 1-layer dummy shape at the table's tile size.
        layers.push(vec![255u8; (tile * tile * 4) as usize]);
    }
    Ok((
        TextureSpec {
            tile_size: tile,
            layers,
            material_layers,
        },
        stats,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materials::MaterialTable;

    #[test]
    fn builds_scene_local_spec_with_remapped_layers() {
        // Tiny voxelizer-shaped fixture: two materials sharing one
        // layer, tile_size 2.
        let dir = std::env::temp_dir().join(format!("engine_tex_{}", std::process::id()));
        let blocks = dir.join("blocks");
        std::fs::create_dir_all(&blocks).unwrap();
        std::fs::write(
            dir.join("blocks.json"),
            r#"{"format":"png-layers","tile_size":2,"layer_count":3}"#,
        )
        .unwrap();
        std::fs::write(
            dir.join("materials.json"),
            r#"[
              {"id":"t_a","resolved_id":"t_a","resolution":"exact","material_index":0,"top_layer":0,"side_layer":1,"bottom_layer":0,"height_voxels":3,"transparency":"opaque","cutout":false,"transparent":false,"tint":[255,255,255],"sub_model":null,"multitile":false,"subtile_layers":{}},
              {"id":"t_b","resolved_id":"t_b","resolution":"exact","material_index":1,"top_layer":2,"side_layer":2,"bottom_layer":2,"height_voxels":3,"transparency":"opaque","cutout":false,"transparent":false,"tint":[255,255,255],"sub_model":null,"multitile":false,"subtile_layers":{}}
            ]"#,
        )
        .unwrap();
        for (i, v) in [(0usize, 10u8), (1, 20), (2, 30)] {
            let img = image::RgbaImage::from_pixel(2, 2, image::Rgba([v, v, v, 255]));
            img.save(blocks.join(format!("layer_{i:04}.png"))).unwrap();
        }
        let table = MaterialTable::load(&dir).unwrap();
        let id_map: HashMap<u32, String> = [(7u32, "t_a".to_string()), (9, "t_b".to_string())]
            .into_iter()
            .collect();
        let (spec, stats) = build_texture_spec(&table, &dir, &id_map).unwrap();
        assert_eq!(stats.textured_materials, 2);
        assert_eq!(stats.missing_materials, 0);
        // t_a uses layers 0,1,0 -> compact 0,1,0; t_b uses 2 -> 2.
        assert_eq!(stats.layers_uploaded, 3);
        assert_eq!(spec.material_layers[7], [0, 1, 0, 1]);
        assert_eq!(spec.material_layers[9], [2, 2, 2, 1]);
        assert_eq!(spec.material_layers[0], [0, 0, 0, 0], "unmapped ids stay flat");
        assert_eq!(spec.layers[1][0], 20, "layer bytes carried through");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
