//! Indexed triangle mesh output, one mesh per render pass.

use std::collections::HashSet;

use crate::block::Pass;

/// An indexed mesh: 4 vertices + 6 indices per quad.
#[derive(Debug, Clone, Default)]
pub struct Mesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 4]>,
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
    pub fn push_quad(&mut self, corners: [[f32; 3]; 4], normal: [f32; 3], color: [f32; 4]) {
        let base = self.positions.len() as u32;
        for c in corners {
            self.positions.push(c);
            self.normals.push(normal);
            self.colors.push(color);
        }
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    pub fn append(&mut self, other: &Mesh) {
        let base = self.positions.len() as u32;
        self.positions.extend_from_slice(&other.positions);
        self.normals.extend_from_slice(&other.normals);
        self.colors.extend_from_slice(&other.colors);
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
        let others = [0, 1, 2].into_iter().filter(|&a| a != axis).collect::<Vec<_>>();
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
