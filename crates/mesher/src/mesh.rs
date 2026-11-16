//! Indexed triangle mesh output, one mesh per render pass.

use std::collections::HashSet;

use crate::block::Pass;

/// Texture-layer sentinel for flat (untextured) vertices (Phase 2f).
/// The raster fragment shader compares against this exact value and
/// falls back to the pre-2f flat shading math.
pub const FLAT_LAYER: u32 = u32::MAX;

/// Stage F macro-layer encoding in a vertex layer word. A top face
/// whose material carries a world-space macro texture (see the
/// voxelizer's materials.json v2) cannot address the scene texture
/// array — the macro lives in a separate 128px array — so its layer
/// word is packed as:
///
/// ```text
/// bit  31     : macro flag
/// bits 24..31 : span_tiles (the macro's world span, in tiles)
/// bits  0..24 : layer index into the macro array
/// ```
///
/// The mesher reads the span back out to emit world-based UVs; the
/// raster fragment shader masks the layer index and samples the
/// macro array. `FLAT_LAYER` (all bits set) also has bit 31 set, so
/// consumers must test for flat *before* testing the macro flag —
/// the mesher never stamps the encoding into a flat vertex, and the
/// shader's flat check comes first.
pub const MACRO_LAYER_BIT: u32 = 1 << 31;
pub const MACRO_LAYER_MASK: u32 = 0x00FF_FFFF;
pub const MACRO_SPAN_SHIFT: u32 = 24;

/// Pack a macro-array layer index + world span into a vertex layer
/// word (see [`MACRO_LAYER_BIT`]).
pub fn encode_macro_layer(layer: u32, span_tiles: u32) -> u32 {
    debug_assert!(layer <= MACRO_LAYER_MASK, "macro layer index too large");
    debug_assert!(
        span_tiles > 0 && span_tiles < 128,
        "macro span must fit in 7 bits"
    );
    MACRO_LAYER_BIT | (span_tiles << MACRO_SPAN_SHIFT) | (layer & MACRO_LAYER_MASK)
}

/// Unpack a vertex layer word: `Some((macro layer, span_tiles))`
/// when the macro flag is set, `None` for ordinary array layers.
/// (Also returns `Some` for `FLAT_LAYER`'s all-ones word — check
/// flat first, as documented on [`MACRO_LAYER_BIT`].)
pub fn decode_macro_layer(word: u32) -> Option<(u32, u32)> {
    if word & MACRO_LAYER_BIT != 0 {
        Some((
            word & MACRO_LAYER_MASK,
            (word >> MACRO_SPAN_SHIFT) & 0x7F,
        ))
    } else {
        None
    }
}

/// An indexed mesh: 4 vertices + 6 indices per quad.
#[derive(Debug, Clone, Default)]
pub struct Mesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 4]>,
    /// Per-vertex UVs (Phase 2f). Parallel to `positions`.
    pub uvs: Vec<[f32; 2]>,
    /// Per-vertex texture-array layer (Phase 2f), `FLAT_LAYER` for
    /// untextured faces. Parallel to `positions`.
    pub layers: Vec<u32>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    pub fn quad_count(&self) -> usize {
        self.indices.len() / 6
    }

    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Append one quad. `corners` must be counter-clockwise seen from the
    /// side `normal` points to (both meshers guarantee this; the winding
    /// test verifies it geometrically).
    ///
    /// Flat variant: default UV quad-order (0,0),(1,0),(1,1),(0,1) and
    /// the `FLAT_LAYER` sentinel — the shader never samples for these.
    pub fn push_quad(&mut self, corners: [[f32; 3]; 4], normal: [f32; 3], color: [f32; 4]) {
        self.push_quad_textured(
            corners,
            normal,
            color,
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            FLAT_LAYER,
        );
    }

    /// Append one textured quad (Phase 2f): per-corner `uvs` in the
    /// same order as `corners`, all sampling texture-array `layer`.
    pub fn push_quad_textured(
        &mut self,
        corners: [[f32; 3]; 4],
        normal: [f32; 3],
        color: [f32; 4],
        uvs: [[f32; 2]; 4],
        layer: u32,
    ) {
        let base = self.positions.len() as u32;
        for (i, c) in corners.into_iter().enumerate() {
            self.positions.push(c);
            self.normals.push(normal);
            self.colors.push(color);
            self.uvs.push(uvs[i]);
            self.layers.push(layer);
        }
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    pub fn append(&mut self, other: &Mesh) {
        let base = self.positions.len() as u32;
        self.positions.extend_from_slice(&other.positions);
        self.normals.extend_from_slice(&other.normals);
        self.colors.extend_from_slice(&other.colors);
        self.uvs.extend_from_slice(&other.uvs);
        self.layers.extend_from_slice(&other.layers);
        self.indices.extend(other.indices.iter().map(|i| i + base));
    }

    /// Total surface area in world units² (sum over quads).
    pub fn surface_area(&self) -> f32 {
        let mut area = 0.0f32;
        for quad in self.indices.chunks_exact(6) {
            let p = |i: u32| self.positions[i as usize];
            let (a, b, c) = (p(quad[0]), p(quad[1]), p(quad[2]));
            let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let cross = [
                ab[1] * ac[2] - ab[2] * ac[1],
                ab[2] * ac[0] - ab[0] * ac[2],
                ab[0] * ac[1] - ab[1] * ac[0],
            ];
            // |ab x ac| is the full quad area (parallelogram), not half:
            // the quad's two triangles tile exactly this parallelogram.
            area += (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
        }
        area
    }
}

/// The three per-pass meshes produced for one chunk/grid.
#[derive(Debug, Clone, Default)]
pub struct PassMeshes {
    pub meshes: [Mesh; 3],
}

impl PassMeshes {
    pub fn get(&self, pass: Pass) -> &Mesh {
        &self.meshes[pass.index()]
    }

    pub fn total_quads(&self) -> usize {
        self.meshes.iter().map(|m| m.quad_count()).sum()
    }

    pub fn total_vertices(&self) -> usize {
        self.meshes.iter().map(|m| m.vertex_count()).sum()
    }

    pub fn total_area(&self) -> f32 {
        self.meshes.iter().map(|m| m.surface_area()).sum()
    }
}

/// Expand every quad of a mesh (assumed built at block_size = 1) back to
/// the set of unit face-cells it covers: `(block cell, direction index)`
/// where direction index is 0:+X 1:-X 2:+Y(south) 3:-Y 4:+Z(up) 5:-Z in
/// grid space. Used by the greedy-vs-naive property test to prove both
/// meshers cover exactly the same visible faces, and (by comparing the
/// expanded count with the set size) that no two quads overlap.
pub fn covered_face_cells(mesh: &Mesh) -> (HashSet<([i32; 3], usize)>, usize) {
    let mut set = HashSet::new();
    let mut expanded = 0usize;
    for quad in mesh.indices.chunks_exact(6) {
        let n = mesh.normals[quad[0] as usize];
        // World normal -> grid normal (inverse of the (x, z, -y) rotation).
        let gn = [n[0], -n[2], n[1]];
        let (axis, dir) = if gn[0] > 0.5 {
            (0, 0)
        } else if gn[0] < -0.5 {
            (0, 1)
        } else if gn[1] > 0.5 {
            (1, 2)
        } else if gn[1] < -0.5 {
            (1, 3)
        } else if gn[2] > 0.5 {
            (2, 4)
        } else {
            (2, 5)
        };
        let others = [0, 1, 2]
            .into_iter()
            .filter(|&a| a != axis)
            .collect::<Vec<_>>();
        // World position -> grid position (inverse rotation).
        let to_grid = |p: [f32; 3]| [p[0], -p[2], p[1]];
        let pts: Vec<[f32; 3]> = quad[..4]
            .iter()
            .map(|&i| to_grid(mesh.positions[i as usize]))
            .collect();
        let plane = pts[0][axis];
        let (u, v) = (others[0], others[1]);
        let u_min = pts.iter().map(|p| p[u]).fold(f32::MAX, f32::min).round() as i32;
        let u_max = pts.iter().map(|p| p[u]).fold(f32::MIN, f32::max).round() as i32;
        let v_min = pts.iter().map(|p| p[v]).fold(f32::MAX, f32::min).round() as i32;
        let v_max = pts.iter().map(|p| p[v]).fold(f32::MIN, f32::max).round() as i32;
        for uu in u_min..u_max {
            for vv in v_min..v_max {
                let mut cell = [0i32; 3];
                // Positive faces sit on the plane at cell+1 along the axis.
                cell[axis] = if dir % 2 == 0 {
                    plane.round() as i32 - 1
                } else {
                    plane.round() as i32
                };
                cell[u] = uu;
                cell[v] = vv;
                set.insert((cell, dir));
                expanded += 1;
            }
        }
    }
    (set, expanded)
}
