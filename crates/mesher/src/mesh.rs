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
/// bit  30     : blend flag (Stage F part 2, see
///               [`MACRO_BLEND_BIT`] for that sub-encoding)
/// bits 24..30 : span_tiles (the macro's world span, in tiles;
///               contractually < 64 so bit 30 stays clear)
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
    // Spans are capped at 63 (not the field's 127) so bit 30 of
    // the span field stays clear for the blend flag
    // ([`MACRO_BLEND_BIT`]): a plain word must never alias a
    // blend word. Real spans are 4 (the voxelizer's 128px macro
    // at 32px tiles); 64 tiles would be a 2048px texture.
    debug_assert!(
        span_tiles > 0 && span_tiles < 64,
        "macro span must fit in 6 bits (bit 30 is the blend flag)"
    );
    MACRO_LAYER_BIT | (span_tiles << MACRO_SPAN_SHIFT) | (layer & MACRO_LAYER_MASK)
}

/// Unpack a vertex layer word: `Some((macro layer, span_tiles))`
/// when the macro flag is set, `None` for ordinary array layers.
/// (Also returns `Some` for `FLAT_LAYER`'s all-ones word — check
/// flat first, as documented on [`MACRO_LAYER_BIT`].) A blend
/// word ([`MACRO_BLEND_BIT`]) unpacks to its *own* layer + span —
/// the only fields the mesher needs (world UVs); the engine's
/// shaders unpack the full blend fields themselves.
pub fn decode_macro_layer(word: u32) -> Option<(u32, u32)> {
    if word & MACRO_LAYER_BIT != 0 {
        if word & MACRO_BLEND_BIT != 0 {
            Some((
                word & MACRO_BLEND_INDEX_MASK,
                (word >> MACRO_SPAN_SHIFT) & 0x3F,
            ))
        } else {
            Some((
                word & MACRO_LAYER_MASK,
                (word >> MACRO_SPAN_SHIFT) & 0x7F,
            ))
        }
    } else {
        None
    }
}

/// Stage F part 2 (border height-blending): a macro top face that
/// borders a *different* macro material carries a blend word —
/// the packed fields the fragment shaders need to run the Stage D
/// height decision against the neighbor material (both macros are
/// world-anchored, so the neighbor is sampled at the same world
/// UV; only its layer index and the border direction travel):
///
/// ```text
/// bit  31     : macro flag ([`MACRO_LAYER_BIT`])
/// bit  30     : blend flag
/// bits 24..30 : span_tiles (6 bits — spans are small: the
///               voxelizer emits 4; plain macro words keep the
///               7-bit field, so a plain word with span >= 64
///               would alias this flag and must never be built)
/// bits 22..24 : border direction (a `FACE_DIRS` cardinal index:
///               0 +X, 1 -X, 2 +Y/south, 3 -Y/north) — the neighbor
///               material lies across that edge of this tile
/// bits 11..22 : neighbor material's macro COLOR layer index
/// bits  0..11 : this material's macro COLOR layer index
/// ```
///
/// Height layers are not packed: in the macro array a material's
/// height layer sits at `color + color_count` (the array's second
/// half), which both shaders already know. The 11-bit index cap
/// (2048 macro color layers per scene) is far above any real
/// scene (one layer per macro material); the engine falls back to
/// the plain macro word when a scene ever exceeds it.
pub const MACRO_BLEND_BIT: u32 = 1 << 30;
pub const MACRO_BLEND_INDEX_MASK: u32 = 0x7FF;
pub const MACRO_BLEND_DIR_SHIFT: u32 = 22;

/// Pack a border-blend layer word (see [`MACRO_BLEND_BIT`]).
pub fn encode_macro_blend_layer(
    own_layer: u32,
    neighbor_layer: u32,
    dir: u32,
    span_tiles: u32,
) -> u32 {
    debug_assert!(
        own_layer <= MACRO_BLEND_INDEX_MASK && neighbor_layer <= MACRO_BLEND_INDEX_MASK,
        "blend macro layer index too large"
    );
    debug_assert!(dir < 4, "blend direction must be a cardinal index");
    debug_assert!(
        span_tiles > 0 && span_tiles < 64,
        "blend macro span must fit in 6 bits"
    );
    MACRO_LAYER_BIT
        | MACRO_BLEND_BIT
        | (span_tiles << MACRO_SPAN_SHIFT)
        | (dir << MACRO_BLEND_DIR_SHIFT)
        | (neighbor_layer << 11)
        | (own_layer & MACRO_BLEND_INDEX_MASK)
}

/// Unpack a blend word: `Some((own layer, neighbor layer, dir,
/// span_tiles))` when both the macro and blend flags are set.
pub fn decode_macro_blend_layer(word: u32) -> Option<(u32, u32, u32, u32)> {
    if word & MACRO_LAYER_BIT != 0 && word & MACRO_BLEND_BIT != 0 {
        Some((
            word & MACRO_BLEND_INDEX_MASK,
            (word >> 11) & MACRO_BLEND_INDEX_MASK,
            (word >> MACRO_BLEND_DIR_SHIFT) & 0x3,
            (word >> MACRO_SPAN_SHIFT) & 0x3F,
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

    /// Append one quad with a PER-CORNER normal (slope shading):
    /// same default UVs / flat layer as [`Mesh::push_quad`].
    pub fn push_quad_per_corner(
        &mut self,
        corners: [[f32; 3]; 4],
        normals: [[f32; 3]; 4],
        color: [f32; 4],
    ) {
        self.push_quad_textured_per_corner(
            corners,
            normals,
            color,
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            FLAT_LAYER,
        );
    }

    /// Append one textured quad with PER-CORNER normals (slope
    /// shading): per-corner `uvs` in the same order as `corners`,
    /// all sampling texture-array `layer`.
    pub fn push_quad_textured_per_corner(
        &mut self,
        corners: [[f32; 3]; 4],
        normals: [[f32; 3]; 4],
        color: [f32; 4],
        uvs: [[f32; 2]; 4],
        layer: u32,
    ) {
        let base = self.positions.len() as u32;
        for (i, c) in corners.into_iter().enumerate() {
            self.positions.push(c);
            self.normals.push(normals[i]);
            self.colors.push(color);
            self.uvs.push(uvs[i]);
            self.layers.push(layer);
        }
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
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

#[cfg(test)]
mod blend_word_tests {
    use super::*;

    #[test]
    fn blend_word_round_trips_and_decodes_as_own_macro() {
        let w = encode_macro_blend_layer(5, 9, 2, 4);
        assert_eq!(decode_macro_blend_layer(w), Some((5, 9, 2, 4)));
        // The mesher's view: a blend word is a macro word for its
        // OWN layer + span (world UV computation).
        assert_eq!(decode_macro_layer(w), Some((5, 4)));
        // A plain macro word is not a blend word (its span is
        // < 64 by the encoding contract, so bit 30 stays clear).
        let plain = encode_macro_layer(7, 63);
        assert_eq!(decode_macro_blend_layer(plain), None);
        assert_eq!(decode_macro_layer(plain), Some((7, 63)));
        // Max field values round-trip.
        let max = encode_macro_blend_layer(2047, 2046, 3, 63);
        assert_eq!(decode_macro_blend_layer(max), Some((2047, 2046, 3, 63)));
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
