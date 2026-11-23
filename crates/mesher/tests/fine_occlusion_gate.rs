//! Stage H gate (8c-A, 2026-11-23): geometric occlusion
//! for fine props — the `docs/phase9-fine.md` §1 candidate,
//! built on Joey's call. The pass is invented lighting
//! (game truth is per-tile `lit_level`); this gate pins its
//! exact, bounded behaviour:
//!
//! - a face in the open keeps factor 1.0 exactly, and an
//!   unoccluded prop meshes with its own colours unchanged;
//! - occluders beside the air cell in front of a face cost
//!   0.11 each (floor contact counts: a lone prop's side
//!   faces at its base read 0.89);
//! - a face boxed in a stamped pocket reads the sum of its
//!   occluders (3 of 4 -> 0.67);
//! - `cell_occlusion` averages the exposed faces (the RT
//!   path's per-cell form).

use mesher::block::Block;
use mesher::chunk::ChunkMap;
use mesher::fine::{
    cell_occlusion, face_occlusion, mesh_chunk_fine, stamp_model, FineLayer, FineSampler,
    ModelCells, FINE,
};
use mesher::scale::WorldScale;
use mesher::Pass;

struct TestModel {
    size: [u32; 3],
    cells: Vec<Option<Block>>,
}

impl TestModel {
    fn new(size: [u32; 3]) -> Self {
        let n = (size[0] * size[1] * size[2]) as usize;
        Self {
            size,
            cells: vec![None; n],
        }
    }
    fn set(&mut self, x: u32, y: u32, z: u32, block: Block) {
        let [w, _h, d] = self.size;
        let i = ((y * d + z) * w + x) as usize;
        self.cells[i] = Some(block);
    }
}

impl ModelCells for TestModel {
    fn size(&self) -> [u32; 3] {
        self.size
    }
    fn cell(&self, x: u32, y: u32, z: u32) -> Option<Block> {
        let [w, h, d] = self.size;
        if x >= w || y >= h || z >= d {
            return None;
        }
        self.cells[((y * d + z) * w + x) as usize]
    }
}

fn blk(id: u32, color: [f32; 4]) -> Block {
    Block {
        id,
        color,
        pass: Pass::Opaque,
        tex_layers: None,
    }
}

const PROP: [f32; 4] = [0.8, 0.5, 0.25, 1.0];

/// Coarse world: chunk [0,0], tile (4,4) has a floor slab
/// (block z 0, fine z 0..32 by expansion), everything else
/// air. A lone prop cell is stamped on the slab at local
/// (20, 20), fine z 32.
fn lone_prop() -> (ChunkMap, FineLayer) {
    let scale = WorldScale::default();
    let mut map = ChunkMap::new(scale);
    map.add_chunk([0, 0], 1);
    map.set_global_block(4, 4, 0, blk(10, [0.1, 0.1, 0.1, 1.0]));
    map.clear_dirty();
    let mut layer = FineLayer::default();
    let mut model = TestModel::new([32, 1, 32]);
    model.set(20, 0, 20, blk(77, PROP));
    stamp_model(&mut layer, (4, 4), 32, &model).unwrap();
    (map, layer)
}

#[test]
fn open_faces_keep_full_light() {
    let (map, layer) = lone_prop();
    let sampler = FineSampler::new(&map, &layer);
    let (fx, fy, fz) = (4 * FINE + 20, 4 * FINE + 20, 32);
    // Top face: nothing beside the air above it.
    assert_eq!(face_occlusion(&sampler, fx, fy, fz, [0, 0, 1]), 1.0);
    // Side faces: the floor slab below the air cell in
    // front is the one occluder (contact shading).
    for dir in [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0]] {
        let f = face_occlusion(&sampler, fx, fy, fz, dir);
        assert!((f - 0.89).abs() < 1e-6, "side face {dir:?} = {f}, want 0.89");
    }
    // Cell form: mean over the 5 exposed faces (the bottom
    // face is not exposed — the slab is under it).
    let c = cell_occlusion(&sampler, fx, fy, fz);
    let want = (1.0 + 4.0 * 0.89) / 5.0;
    assert!((c - want).abs() < 1e-6, "cell occlusion {c}, want {want}");
}

#[test]
fn pocket_face_counts_its_occluders() {
    let scale = WorldScale::default();
    let mut map = ChunkMap::new(scale);
    map.add_chunk([0, 0], 1);
    map.set_global_block(4, 4, 0, blk(10, [0.1, 0.1, 0.1, 1.0]));
    map.clear_dirty();
    let mut layer = FineLayer::default();
    // Three stamped cells forming a pocket around the air
    // cell (21, 20, 32) in front of the first cell's +x
    // face: one across (+x of the air cell), one above it,
    // and the slab below it.
    let mut model = TestModel::new([32, 2, 32]);
    model.set(20, 0, 20, blk(77, PROP)); // the face's owner, fine (148, 148, 32)
    model.set(22, 0, 20, blk(77, PROP)); // across the air cell
    model.set(21, 1, 20, blk(77, PROP)); // above the air cell (fine z 33)
    stamp_model(&mut layer, (4, 4), 32, &model).unwrap();
    let sampler = FineSampler::new(&map, &layer);
    let f = face_occlusion(&sampler, 148, 148, 32, [1, 0, 0]);
    assert!((f - 0.67).abs() < 1e-6, "pocket face = {f}, want 0.67 (3 occluders)");
}

#[test]
fn mesher_applies_the_factor_per_face() {
    let (map, layer) = lone_prop();
    let scale = WorldScale::default();
    let meshes = mesh_chunk_fine(&map, &layer, [0, 0], scale).expect("chunk meshes");
    let opaque = &meshes.meshes[Pass::Opaque.index()];
    let mut saw_plain = false;
    let mut saw_shaded = false;
    for c in &opaque.colors {
        if *c == PROP {
            saw_plain = true;
        }
        if (c[0] - PROP[0] * 0.89).abs() < 1e-6
            && (c[1] - PROP[1] * 0.89).abs() < 1e-6
            && (c[2] - PROP[2] * 0.89).abs() < 1e-6
            && c[3] == 1.0
        {
            saw_shaded = true;
        }
    }
    assert!(saw_plain, "the top face keeps the prop's own colour (factor 1.0)");
    assert!(saw_shaded, "the side faces carry colour x 0.89 (floor contact)");
}

