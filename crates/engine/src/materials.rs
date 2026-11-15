//! Minimal Phase 5 integration: load the voxelizer's `materials.json`
//! + `blocks.json` so the engine can resolve a game id to its block
//! texture layers, transparency pass and height.
//!
//! Scope note: the mesher's `Block`/`Vertex` types (crates/mesher,
//! read-only for Phase 5) carry flat colors and no UV/texture-layer
//! field, so textured *rendering* needs a mesher+pipeline change that
//! belongs to a follow-up. What lands here is the data half: parsing,
//! validation and id -> material lookup, unit-tested, plus the
//! headless-capture regression proving the Phase 2 renderer is
//! untouched.

use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub id: String,
    pub resolved_id: String,
    pub resolution: String,
    pub top_layer: usize,
    pub side_layer: usize,
    pub bottom_layer: usize,
    pub height_voxels: u32,
    pub transparency: String,
    pub cutout: bool,
    pub transparent: bool,
}

#[derive(Debug, Default)]
pub struct MaterialTable {
    pub tile_size: u32,
    pub layer_count: usize,
    by_id: BTreeMap<String, Material>,
}

impl MaterialTable {
    pub fn load(dir: &Path) -> Result<Self, String> {
        let blocks: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(dir.join("blocks.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let mats: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(dir.join("materials.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let mut table = MaterialTable {
            tile_size: blocks["tile_size"].as_u64().unwrap_or(32) as u32,
            layer_count: blocks["layer_count"].as_u64().unwrap_or(0) as usize,
            by_id: BTreeMap::new(),
        };
        for m in mats.as_array().cloned().unwrap_or_default() {
            let get = |k: &str| m[k].as_u64().unwrap_or(0) as usize;
            let mat = Material {
                id: m["id"].as_str().unwrap_or("").to_string(),
                resolved_id: m["resolved_id"].as_str().unwrap_or("").to_string(),
                resolution: m["resolution"].as_str().unwrap_or("").to_string(),
                top_layer: get("top_layer"),
                side_layer: get("side_layer"),
                bottom_layer: get("bottom_layer"),
                height_voxels: m["height_voxels"].as_u64().unwrap_or(0) as u32,
                transparency: m["transparency"].as_str().unwrap_or("opaque").to_string(),
                cutout: m["cutout"].as_bool().unwrap_or(false),
                transparent: m["transparent"].as_bool().unwrap_or(false),
            };
            if mat.top_layer >= table.layer_count && table.layer_count > 0 {
                return Err(format!("material {} layer out of range", mat.id));
            }
            table.by_id.insert(mat.id.clone(), mat);
        }
        Ok(table)
    }
    pub fn get(&self, id: &str) -> Option<&Material> {
        self.by_id.get(id)
    }
    pub fn len(&self) -> usize {
        self.by_id.len()
    }
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }
    /// Render pass name for a material, matching mesher::Pass naming.
    pub fn pass_of(m: &Material) -> &'static str {
        if m.transparent {
            "Transparent"
        } else if m.cutout {
            "Cutout"
        } else {
            "Opaque"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_voxelizer_output() {
        // Build a tiny voxelizer output in a temp dir via JSON literals
        // shaped exactly like the voxelizer's materials.json/blocks.json.
        let dir = std::env::temp_dir().join(format!("engine_mat_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("blocks.json"),
            r#"{"format":"png-layers","tile_size":32,"layer_count":3}"#,
        )
        .unwrap();
        std::fs::write(
            dir.join("materials.json"),
            r#"[{"id":"t_wall","resolved_id":"t_wall","resolution":"exact","material_index":0,"top_layer":0,"side_layer":1,"bottom_layer":2,"height_voxels":3,"transparency":"opaque","cutout":false,"transparent":false,"tint":[255,255,255],"sub_model":null,"multitile":true,"subtile_layers":{}}]"#,
        )
        .unwrap();
        let t = MaterialTable::load(&dir).unwrap();
        assert_eq!(t.len(), 1);
        assert_eq!(t.get("t_wall").unwrap().height_voxels, 3);
        assert_eq!(MaterialTable::pass_of(t.get("t_wall").unwrap()), "Opaque");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
