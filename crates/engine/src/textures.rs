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
//!
//! Phase 2d extends the builder with multitile variants:
//! [`build_texture_spec_multitile`] additionally emits one entry
//! per `treebuild::connect::VariantAssignment` — the virtual
//! material ids `assign_variants` rewrote connected voxels to —
//! whose side layer is the mask-selected subtile layer
//! (`Material::side_layer_for_mask`). Both builders share the one
//! layer-loading/compaction path below, so a layer's compact index
//! means the same thing in base and variant entries.

use std::collections::HashMap;
use std::path::Path;

use treebuild::connect::VariantAssignment;
use treebuild::render::TextureSpec;

use crate::materials::{Material, MaterialTable};

/// What a spec build resolved, for capture reporting and gates.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct TextureStats {
    /// Materials (of those requested) that got a texture entry.
    pub textured_materials: usize,
    /// Requested materials with no voxelizer entry (stay flat).
    pub missing_materials: usize,
    /// Layers uploaded into the scene texture array.
    pub layers_uploaded: usize,
    /// Phase 2d variant entries emitted (multitile builds only).
    pub variants: usize,
    /// Stage F: materials given a world-space macro entry (v2
    /// voxelizer output only; always 0 for v1 dirs).
    pub macros: usize,
}

/// Scene-local layer loading + compaction, shared by the base and
/// multitile spec builders: a global voxelizer layer index maps to
/// one compact array index, decoded at most once.
struct SpecBuilder<'a> {
    tile: u32,
    dir: &'a Path,
    compact_of: HashMap<usize, u32>,
    layers: Vec<Vec<u8>>,
    /// Stage F: the separate macro layer set (see
    /// `treebuild::render::TextureSpec`): color layers compacted
    /// into `macro_colors`, height layers into `macro_heights`;
    /// `finish` concatenates them (colors first).
    macro_size: u32,
    macro_compact_of: HashMap<(usize, bool), u32>,
    macro_colors: Vec<Vec<u8>>,
    macro_heights: Vec<Vec<u8>>,
}

impl<'a> SpecBuilder<'a> {
    fn new(table: &MaterialTable, dir: &'a Path) -> Self {
        Self {
            tile: table.tile_size,
            dir,
            compact_of: HashMap::new(),
            layers: Vec::new(),
            macro_size: table.macro_size,
            macro_compact_of: HashMap::new(),
            macro_colors: Vec::new(),
            macro_heights: Vec::new(),
        }
    }

    /// Stage F: load one macro layer PNG (`macro_<layer>_color.png`
    /// or `macro_<layer>_height.png` beside materials.json) into
    /// its half of the macro set, compacted like block layers.
    /// Height PNGs are grayscale; `to_rgba8` replicates the gray
    /// into RGB so the array format is uniform.
    fn load_macro_layer(&mut self, src: usize, height: bool) -> Result<u32, String> {
        if let Some(&c) = self.macro_compact_of.get(&(src, height)) {
            return Ok(c);
        }
        let kind = if height { "height" } else { "color" };
        let path = self.dir.join(format!("macro_{src:04}_{kind}.png"));
        let img = image::open(&path)
            .map_err(|e| format!("{}: {e}", path.display()))?
            .to_rgba8();
        if img.width() != self.macro_size || img.height() != self.macro_size {
            return Err(format!(
                "{}: {}x{} but macro_size is {}",
                path.display(),
                img.width(),
                img.height(),
                self.macro_size
            ));
        }
        let half = if height {
            &mut self.macro_heights
        } else {
            &mut self.macro_colors
        };
        let idx = half.len() as u32;
        half.push(img.into_raw());
        self.macro_compact_of.insert((src, height), idx);
        Ok(idx)
    }

    fn load_layer(&mut self, src: usize) -> Result<u32, String> {
        if let Some(&c) = self.compact_of.get(&src) {
            return Ok(c);
        }
        let path = self.dir.join("blocks").join(format!("layer_{src:04}.png"));
        let img = image::open(&path)
            .map_err(|e| format!("{}: {e}", path.display()))?
            .to_rgba8();
        if img.width() != self.tile || img.height() != self.tile {
            return Err(format!(
                "{}: {}x{} but tile_size is {}",
                path.display(),
                img.width(),
                img.height(),
                self.tile
            ));
        }
        let idx = self.layers.len() as u32;
        self.layers.push(img.into_raw());
        self.compact_of.insert(src, idx);
        Ok(idx)
    }

    /// The `(top, side, bottom, present)` entry for `mat` with an
    /// explicitly chosen *global* side layer (the base builder
    /// passes `mat.side_layer`; the multitile builder passes the
    /// mask-selected layer).
    fn entry(&mut self, mat: &Material, side_layer_global: usize) -> Result<[u32; 4], String> {
        let top = self.load_layer(mat.top_layer)?;
        let side = self.load_layer(side_layer_global)?;
        let bottom = self.load_layer(mat.bottom_layer)?;
        Ok([top, side, bottom, 1])
    }

    /// Base entries for every material in `id_map`, indexed by
    /// numeric id. Unresolvable materials are counted in `stats`
    /// and left as the all-zero (flat) entry. Also returns the
    /// Stage F macro entries (same indexing; all-zero = no macro).
    fn build_base(
        &mut self,
        table: &MaterialTable,
        id_map: &HashMap<u32, String>,
        stats: &mut TextureStats,
    ) -> (Vec<[u32; 4]>, Vec<[u32; 4]>) {
        let max_id = id_map.keys().copied().max().unwrap_or(0);
        let mut material_layers = vec![[0u32; 4]; (max_id + 1) as usize];
        let mut material_macros = vec![[0u32; 4]; (max_id + 1) as usize];
        // Deterministic order (sorted ids) so specs are reproducible.
        let mut ids: Vec<u32> = id_map.keys().copied().collect();
        ids.sort_unstable();
        for num_id in ids {
            let game_id = &id_map[&num_id];
            let Some(mat) = table.get(game_id) else {
                stats.missing_materials += 1;
                continue;
            };
            match self.entry(mat, mat.side_layer) {
                Ok(entry) => {
                    material_layers[num_id as usize] = entry;
                    stats.textured_materials += 1;
                }
                Err(e) => {
                    eprintln!("voxel-engine: material {game_id} stays flat: {e}");
                    stats.missing_materials += 1;
                }
            }
            // Stage F: the material's macro, when the v2 table
            // carries one. A macro that fails to load leaves the
            // material on its per-tile top face (v1 behavior).
            if let Some(m) = mat.macro_ {
                let loaded = self
                    .load_macro_layer(m.color_layer, false)
                    .and_then(|c| Ok((c, self.load_macro_layer(m.height_layer, true)?)));
                match loaded {
                    Ok((color, height)) => {
                        material_macros[num_id as usize] =
                            [color, height, m.span_tiles, 1];
                        stats.macros += 1;
                    }
                    Err(e) => {
                        eprintln!("voxel-engine: material {game_id} macro skipped: {e}");
                    }
                }
            }
        }
        (material_layers, material_macros)
    }

    fn finish(
        self,
        material_layers: Vec<[u32; 4]>,
        mut material_macros: Vec<[u32; 4]>,
        stats: &mut TextureStats,
    ) -> TextureSpec {
        let mut layers = self.layers;
        stats.layers_uploaded = layers.len();
        if layers.is_empty() {
            // TextureSpec requires at least one layer; give it the
            // 1-layer dummy shape at the table's tile size.
            layers.push(vec![255u8; (self.tile * self.tile * 4) as usize]);
        }
        // Stage F: macro entries are indexed like material_layers;
        // pad to the final length (variant builders may have grown
        // material_layers after the base pass), then join the two
        // macro halves: colors first, heights offset by the color
        // count.
        material_macros.resize(material_layers.len(), [0, 0, 0, 0]);
        let color_count = self.macro_colors.len() as u32;
        for entry in material_macros.iter_mut() {
            if entry[3] != 0 {
                entry[1] += color_count;
            }
        }
        let mut macro_layers = self.macro_colors;
        macro_layers.extend(self.macro_heights);
        TextureSpec {
            tile_size: self.tile,
            layers,
            material_layers,
            macro_size: if macro_layers.is_empty() {
                0
            } else {
                self.macro_size
            },
            macro_layers,
            material_macros,
        }
    }
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
    let mut stats = TextureStats::default();
    let mut builder = SpecBuilder::new(table, dir);
    let (material_layers, material_macros) = builder.build_base(table, id_map, &mut stats);
    Ok((builder.finish(material_layers, material_macros, &mut stats), stats))
}

/// Phase 2d: the base spec plus one variant entry per assignment.
/// Each variant keeps its base material's top/bottom layers and
/// takes the side layer `Material::side_layer_for_mask` selects for
/// the assignment's connection mask, so a voxel the world builder
/// rewrote to `virtual_id` samples its connected-wall picture.
/// Assignments whose base material doesn't resolve are skipped
/// (the base pass already counted them missing; their voxels shade
/// flat through the all-zero entry).
pub fn build_texture_spec_multitile(
    table: &MaterialTable,
    dir: &Path,
    id_map: &HashMap<u32, String>,
    assignments: &[VariantAssignment],
) -> Result<(TextureSpec, TextureStats), String> {
    let mut stats = TextureStats::default();
    let mut builder = SpecBuilder::new(table, dir);
    let (mut material_layers, material_macros) = builder.build_base(table, id_map, &mut stats);
    let mut sorted: Vec<&VariantAssignment> = assignments.iter().collect();
    sorted.sort_by_key(|a| a.virtual_id);
    for a in sorted {
        let Some(game_id) = id_map.get(&a.base_id) else {
            continue;
        };
        let Some(mat) = table.get(game_id) else {
            continue;
        };
        let side = mat.side_layer_for_mask(a.mask);
        match builder.entry(mat, side) {
            Ok(entry) => {
                let idx = a.virtual_id as usize;
                if idx >= material_layers.len() {
                    material_layers.resize(idx + 1, [0, 0, 0, 0]);
                }
                material_layers[idx] = entry;
                stats.variants += 1;
            }
            Err(e) => {
                eprintln!(
                    "voxel-engine: variant (base {}, mask {}) stays flat: {e}",
                    a.base_id, a.mask
                );
            }
        }
    }
    Ok((builder.finish(material_layers, material_macros, &mut stats), stats))
}

/// One state-variant allocation (Phase 2d follow-up): voxels
/// carrying state (`open` / `broken`) for base material
/// `base_id` are rewritten to `virtual_id` — the state analogue
/// of `treebuild::connect::VariantAssignment`'s (base, mask)
/// pairs. The consumer is the vehicle renderer: a vpart's state
/// comes from its exported flags, never from its neighbors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StateVariantAssignment {
    pub base_id: u32,
    pub virtual_id: u32,
    pub open: bool,
    pub broken: bool,
}

/// The base spec plus one state-variant entry per assignment.
/// Each variant keeps its base material's top/bottom layers and
/// takes the side layer `Material::side_layer_for_state`
/// selects, so a stateful part (an open vehicle door) samples
/// its state picture. Assignments whose base material doesn't
/// resolve are skipped, like the multitile builder's.
pub fn build_texture_spec_state_variants(
    table: &MaterialTable,
    dir: &Path,
    id_map: &HashMap<u32, String>,
    assignments: &[StateVariantAssignment],
) -> Result<(TextureSpec, TextureStats), String> {
    let mut stats = TextureStats::default();
    let mut builder = SpecBuilder::new(table, dir);
    let (mut material_layers, material_macros) = builder.build_base(table, id_map, &mut stats);
    let mut sorted: Vec<&StateVariantAssignment> = assignments.iter().collect();
    sorted.sort_by_key(|a| a.virtual_id);
    for a in sorted {
        let Some(game_id) = id_map.get(&a.base_id) else {
            continue;
        };
        let Some(mat) = table.get(game_id) else {
            continue;
        };
        let side = mat.side_layer_for_state(a.open, a.broken);
        match builder.entry(mat, side) {
            Ok(entry) => {
                let idx = a.virtual_id as usize;
                if idx >= material_layers.len() {
                    material_layers.resize(idx + 1, [0, 0, 0, 0]);
                }
                material_layers[idx] = entry;
                stats.variants += 1;
            }
            Err(e) => {
                eprintln!(
                    "voxel-engine: state variant (base {}, open {}, broken {}) stays flat: {e}",
                    a.base_id, a.open, a.broken
                );
            }
        }
    }
    Ok((builder.finish(material_layers, material_macros, &mut stats), stats))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materials::MaterialTable;

    fn fixture(dir: &std::path::Path) {
        // Tiny voxelizer-shaped fixture: two materials sharing one
        // layer, tile_size 2. t_a is multitile with two subtile
        // layers (edge = 3, unconnected = 4) beyond the base three.
        let blocks = dir.join("blocks");
        std::fs::create_dir_all(&blocks).unwrap();
        std::fs::write(
            dir.join("blocks.json"),
            r#"{"format":"png-layers","tile_size":2,"layer_count":6}"#,
        )
        .unwrap();
        std::fs::write(
            dir.join("materials.json"),
            r#"[
              {"id":"t_a","resolved_id":"t_a","resolution":"exact","material_index":0,"top_layer":0,"side_layer":1,"bottom_layer":0,"height_voxels":3,"transparency":"opaque","cutout":false,"transparent":false,"tint":[255,255,255],"sub_model":null,"multitile":true,"subtile_layers":{"edge":3,"unconnected":4,"open":5}},
              {"id":"t_b","resolved_id":"t_b","resolution":"exact","material_index":1,"top_layer":2,"side_layer":2,"bottom_layer":2,"height_voxels":3,"transparency":"opaque","cutout":false,"transparent":false,"tint":[255,255,255],"sub_model":null,"multitile":false,"subtile_layers":{}}
            ]"#,
        )
        .unwrap();
        for (i, v) in [(0usize, 10u8), (1, 20), (2, 30), (3, 40), (4, 50), (5, 60)] {
            let img = image::RgbaImage::from_pixel(2, 2, image::Rgba([v, v, v, 255]));
            img.save(blocks.join(format!("layer_{i:04}.png"))).unwrap();
        }
    }

    #[test]
    fn builds_scene_local_spec_with_remapped_layers() {
        let dir = std::env::temp_dir().join(format!("engine_tex_{}", std::process::id()));
        fixture(&dir);
        let table = MaterialTable::load(&dir).unwrap();
        let id_map: HashMap<u32, String> = [(7u32, "t_a".to_string()), (9, "t_b".to_string())]
            .into_iter()
            .collect();
        let (spec, stats) = build_texture_spec(&table, &dir, &id_map).unwrap();
        assert_eq!(stats.textured_materials, 2);
        assert_eq!(stats.missing_materials, 0);
        assert_eq!(stats.variants, 0);
        // t_a uses layers 0,1,0 -> compact 0,1,0; t_b uses 2 -> 2.
        assert_eq!(stats.layers_uploaded, 3);
        assert_eq!(spec.material_layers[7], [0, 1, 0, 1]);
        assert_eq!(spec.material_layers[9], [2, 2, 2, 1]);
        assert_eq!(
            spec.material_layers[0],
            [0, 0, 0, 0],
            "unmapped ids stay flat"
        );
        assert_eq!(spec.layers[1][0], 20, "layer bytes carried through");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn multitile_spec_adds_variant_entries_with_selected_side_layers() {
        let dir = std::env::temp_dir().join(format!("engine_tex_mt_{}", std::process::id()));
        fixture(&dir);
        let table = MaterialTable::load(&dir).unwrap();
        let id_map: HashMap<u32, String> = [(7u32, "t_a".to_string()), (9, "t_b".to_string())]
            .into_iter()
            .collect();
        // As assign_variants would emit for base 7: an E+W run
        // middle (mask 10 -> edge, global layer 3), an isolated
        // voxel (mask 0 -> unconnected, global layer 4), and a
        // corner (mask 3 -> subtile absent from the fixture, so the
        // variant falls back to the base side layer).
        let assignments = [
            VariantAssignment {
                base_id: 7,
                mask: 10,
                virtual_id: 20,
            },
            VariantAssignment {
                base_id: 7,
                mask: 0,
                virtual_id: 21,
            },
            VariantAssignment {
                base_id: 7,
                mask: 3,
                virtual_id: 22,
            },
        ];
        let (spec, stats) =
            build_texture_spec_multitile(&table, &dir, &id_map, &assignments).unwrap();
        assert_eq!(stats.variants, 3);
        // Base entries unchanged.
        assert_eq!(spec.material_layers[7], [0, 1, 0, 1]);
        // Variants: top/bottom are the base's compact layers (0, 0);
        // sides are the compact indices of global layers 3, 4, and
        // (fallback) 1, loaded in assignment order after the bases'
        // three layers: 3 -> compact 3, 4 -> compact 4.
        assert_eq!(spec.material_layers[20], [0, 3, 0, 1], "edge variant");
        assert_eq!(
            spec.material_layers[21],
            [0, 4, 0, 1],
            "unconnected variant"
        );
        assert_eq!(
            spec.material_layers[22],
            [0, 1, 0, 1],
            "corner falls back to base side"
        );
        assert_eq!(spec.layers[3][0], 40, "edge layer bytes");
        assert_eq!(spec.layers[4][0], 50, "unconnected layer bytes");
        assert_eq!(stats.layers_uploaded, 5);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn state_variant_spec_selects_open_broken_side_layers() {
        let dir = std::env::temp_dir().join(format!("engine_tex_state_{}", std::process::id()));
        fixture(&dir);
        let table = MaterialTable::load(&dir).unwrap();
        let id_map: HashMap<u32, String> = [(7u32, "t_a".to_string())].into_iter().collect();
        // An open part (virtual 30) and a broken part (virtual
        // 31). The fixture entry carries an "open" subtile layer
        // (global 5) but no "broken" layer, so broken falls back
        // to the base side — the same fallback rule as masks.
        let assignments = [
            StateVariantAssignment {
                base_id: 7,
                virtual_id: 30,
                open: true,
                broken: false,
            },
            StateVariantAssignment {
                base_id: 7,
                virtual_id: 31,
                open: false,
                broken: true,
            },
            // Open AND broken: the game's ternary order picks
            // open (pinned cata_tiles.cpp vpart paths).
            StateVariantAssignment {
                base_id: 7,
                virtual_id: 32,
                open: true,
                broken: true,
            },
        ];
        let (spec, stats) =
            build_texture_spec_state_variants(&table, &dir, &id_map, &assignments).unwrap();
        assert_eq!(stats.variants, 3);
        // Base loads layers 0,1 (top/side) -> compact 0,1. The
        // open variant then loads global 5 -> compact 2.
        assert_eq!(spec.material_layers[30], [0, 2, 0, 1], "open variant side = open layer");
        assert_eq!(spec.layers[2][0], 60, "open layer bytes");
        assert_eq!(
            spec.material_layers[31],
            [0, 1, 0, 1],
            "broken with no broken layer falls back to base side"
        );
        assert_eq!(
            spec.material_layers[32],
            [0, 2, 0, 1],
            "open+broken selects open (game's ternary order)"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
