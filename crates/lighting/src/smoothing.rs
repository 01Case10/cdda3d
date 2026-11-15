//! Gates 7.3 + 7.4 — no light leaking *and* smoothing. These two
//! gates constrain each other, so the scheme is designed for both at
//! once and documented here in full.
//!
//! ## The scheme: air-side face sampling with in-plane vertex averaging
//!
//! Light is per-tile game data. This crate **never recomputes FOV and
//! never propagates light engine-side** — no flood fill, no blur over
//! the tile grid. All smoothing is *sampling* of the game's values at
//! face/vertex positions, under three rules:
//!
//! 1. **A face takes its light from the air side.** Every visible face
//!    lies between a solid tile `S` and the tile in front of the face,
//!    `A = S + normal`. The face's centre light is `A`'s game light —
//!    full stop. A wall between a lit room and a dark room therefore
//!    has a bright face toward the lit room and a dark face toward the
//!    dark room, from the *same* wall block: the two sides never mix,
//!    because neither side's sample ever reads across the wall plane.
//!    (Per-block baking cannot express this — a `Block` has one color —
//!    which is why the precise path is the per-vertex baker in
//!    `bake.rs`; the per-block baker is documented there as the
//!    tile-centre approximation.)
//! 2. **A vertex averages only air-side, in-plane neighbours.** A face
//!    vertex touches up to 4 tiles *in the plane parallel to the face,
//!    on the air side of it* (offsets `0 / -1` along the face's two
//!    in-plane axes from `A`). The vertex light is the mean of those
//!    tiles' raw samples, **skipping** any tile that is occluding
//!    (a wall tile is never an air-side light source), `Hidden`
//!    (never-seen light must not bleed into a seen face), or
//!    `Memorized` when the centre is `Visible` and vice versa
//!    (the memory boundary stays a boundary). Because the sampler is
//!    the whole stored grid — the same cross-chunk pattern the mesher
//!    uses via `VoxelSampler` — vertices on a chunk border average
//!    tiles from *both* chunks, so a gradient crossing a border is
//!    interpolated identically to one inside a chunk: no seam.
//! 3. **Top/bottom faces** follow the same rule with the air side
//!    above/below: a floor slab's top face samples the tile itself
//!    (the floor tile's airspace is its own tile — the slab occupies
//!    only the bottom block of its z-level) averaged in-plane over
//!    its horizontal neighbours, again skipping occluders/hidden.
//!    In code this falls out of the general rule if the caller passes
//!    the floor tile as both `S` and `A` for its top face; the helper
//!    [`top_face_vertex_light`] does exactly that.
//!
//! What this does *not* do: a 1-tile-thick diagonal corner can still
//! show the average of diagonally-touching air tiles at the shared
//! vertex — that is interpolation *within* the air side, not leaking
//! through a wall, and it is bounded to one vertex.

use crate::grid::LightSampler;
use crate::mapping::{LightSample, Visibility};

/// Face directions in grid space, same order as `mesher`'s
/// `FACE_DIRS`: 0:+X 1:-X 2:+Y(south) 3:-Y(north) 4:+Z(up) 5:-Z(down).
pub const FACE_DIRS: [[i32; 3]; 6] = [
    [1, 0, 0],
    [-1, 0, 0],
    [0, 1, 0],
    [0, -1, 0],
    [0, 0, 1],
    [0, 0, -1],
];

fn axis_of(dir: [i32; 3]) -> usize {
    (0..3).find(|&a| dir[a] != 0).unwrap()
}

/// Centre light of the face of solid tile `solid` in direction `di`
/// (an index into [`FACE_DIRS`]). Rule 1 only — no averaging.
pub fn face_light<S: LightSampler + ?Sized>(s: &S, solid: [i32; 3], di: usize) -> LightSample {
    let d = FACE_DIRS[di];
    let air = [solid[0] + d[0], solid[1] + d[1], solid[2] + d[2]];
    air_side_sample(s, air)
}

/// The presented-aware sample of a single air-side tile: `Hidden`
/// reads black, `Memorized` reads its (unsmoothed) presented sample,
/// `Visible` reads its raw sample. Smoothing callers that average
/// must use raw samples + a visibility decision instead — see
/// [`vertex_light`], which returns both.
pub fn air_side_sample<S: LightSampler + ?Sized>(s: &S, air: [i32; 3]) -> LightSample {
    let t = s.tile_light(air[0], air[1], air[2]);
    match t.visibility() {
        Visibility::Hidden => LightSample::BLACK,
        Visibility::Memorized => crate::mapping::presented_sample(t),
        Visibility::Visible => t.raw_sample(),
    }
}

/// Vertex light for one vertex of the face of `solid` in direction
/// `di`. `corner` selects the vertex: its two entries are `0` or `1`
/// along the face's in-plane axes (ordered as the axes appear, low
/// axis first) — e.g. for a `+X` face the in-plane axes are `y, z`,
/// so `corner = [0, 1]` is the low-y / high-z vertex.
///
/// Returns `(sample, visibility)`: for `Visible` faces `sample` is
/// the smoothed raw average (multiply the albedo by it); for
/// `Memorized` faces `sample` is the centre tile's presented sample
/// and the caller must *also* desaturate the albedo (see
/// `bake::shade_vertex` for the reference composition); for `Hidden`
/// the sample is black.
pub fn vertex_light<S: LightSampler + ?Sized>(
    s: &S,
    solid: [i32; 3],
    di: usize,
    corner: [i32; 2],
) -> (LightSample, Visibility) {
    let d = FACE_DIRS[di];
    let air = [solid[0] + d[0], solid[1] + d[1], solid[2] + d[2]];
    vertex_light_from_air(s, air, di, corner)
}

/// [`vertex_light`] starting from the air-side tile directly — the
/// form the mesh baker uses, since a mesh vertex knows its air tile
/// (solid tile + normal) more readily than the solid tile alone.
pub fn vertex_light_from_air<S: LightSampler + ?Sized>(
    s: &S,
    air: [i32; 3],
    di: usize,
    corner: [i32; 2],
) -> (LightSample, Visibility) {
    let centre = s.tile_light(air[0], air[1], air[2]);
    let vis = centre.visibility();
    if vis != Visibility::Visible {
        // No smoothing across the memory / hidden boundary (rule 2).
        return (air_side_sample(s, air), vis);
    }
    let d = FACE_DIRS[di];
    let axis = axis_of(d);
    let in_plane: Vec<usize> = (0..3).filter(|&a| a != axis).collect();
    // Reconstruct the vertex's grid corner from the face-local
    // corner, then delegate to the precise grid-corner form.
    let mut grid_corner = air;
    // The face plane sits between solid and air: along `axis` the
    // corner coordinate is air's coordinate when the face normal
    // points negative (face is air's low side) and air+1 when it
    // points positive (face is solid's high side = plane at solid+1).
    grid_corner[axis] = if d[axis] > 0 {
        air[axis]
    } else {
        air[axis] + 1
    };
    // Wait: for d>0, solid = air-1, plane = solid+1 = air. For d<0,
    // solid = air+1, plane = solid = air+1. Correct as written.
    for (k, &ax) in in_plane.iter().enumerate() {
        grid_corner[ax] = air[ax] + corner[k];
    }
    vertex_light_at_grid_corner(s, air, di, grid_corner)
}

/// Precise vertex light: `grid_corner` is the vertex position in
/// *tile-grid corner coordinates* (integers; tile `(x,y,z)` spans
/// corners `[x, x+1] x [y, y+1] x [z, z+1]`). The tiles touching the
/// vertex on the air side are, per in-plane axis, `corner-1` and
/// `corner` — intersected with the air side of the face plane.
/// This is the function the mesh baker calls.
pub fn vertex_light_at_grid_corner<S: LightSampler + ?Sized>(
    s: &S,
    air: [i32; 3],
    di: usize,
    grid_corner: [i32; 3],
) -> (LightSample, Visibility) {
    let centre = s.tile_light(air[0], air[1], air[2]);
    let vis = centre.visibility();
    if vis != Visibility::Visible {
        return (air_side_sample(s, air), vis);
    }
    let d = FACE_DIRS[di];
    let axis = axis_of(d);
    let in_plane: Vec<usize> = (0..3).filter(|&a| a != axis).collect();
    let mut samples: Vec<LightSample> = Vec::with_capacity(4);
    // Top/bottom faces (axis == z): average the 2x2 tiles touching
    // the vertex in the horizontal plane. Side faces: light does not
    // vary within a z-level (it is per-tile data), so averaging
    // "vertically" would mix *floors* — instead average only the 2
    // tiles touching the vertex along the horizontal in-plane axis,
    // at the air tile's own level.
    let offsets: Vec<[i32; 2]> = if axis == 2 {
        vec![[0, 0], [0, -1], [-1, 0], [-1, -1]]
    } else {
        vec![[0, 0], [-1, 0]]
    };
    for off in offsets {
        let mut p = air;
        p[in_plane[0]] = grid_corner[in_plane[0]] + off[0];
        if axis == 2 {
            p[in_plane[1]] = grid_corner[in_plane[1]] + off[1];
        }
        // For side faces in_plane[1] is z: keep the air tile's level.
        p[axis] = air[axis];
        let t = s.tile_light(p[0], p[1], p[2]);
        if t.occluding {
            continue; // a wall tile is never an air-side source
        }
        if t.visibility() != Visibility::Visible {
            continue; // hidden/memorized never bleed into visible
        }
        samples.push(t.raw_sample());
    }
    if samples.is_empty() {
        // The centre itself was filtered (cannot happen for a
        // non-occluding centre, but an occluding *air* tile means the
        // face is internal — fall back to the centre raw sample).
        return (centre.raw_sample(), vis);
    }
    (LightSample::average(&samples), vis)
}

/// Top-face vertex light for a floor tile: the air side is the tile
/// itself, in-plane axes are `x, y` at the tile's own z. Equivalent
/// to calling [`vertex_light_at_grid_corner`] with `di = 4 (+Z)` and
/// `air = tile`, provided the caller treats the slab's solid tile as
/// the tile *below* the airspace — for a floor slab there is no tile
/// below in light terms, so this helper encodes rule 3 directly:
/// average the tile and its horizontal neighbours touching the
/// vertex, skipping occluders/hidden/memorized mismatches.
pub fn top_face_vertex_light<S: LightSampler + ?Sized>(
    s: &S,
    tile: [i32; 3],
    grid_corner_xy: [i32; 2],
) -> (LightSample, Visibility) {
    let centre = s.tile_light(tile[0], tile[1], tile[2]);
    let vis = centre.visibility();
    if vis != Visibility::Visible {
        return (air_side_sample(s, tile), vis);
    }
    let mut samples = Vec::with_capacity(4);
    for &ox in &[0, -1] {
        for &oy in &[0, -1] {
            let p = [grid_corner_xy[0] + ox, grid_corner_xy[1] + oy, tile[2]];
            let t = s.tile_light(p[0], p[1], p[2]);
            if t.occluding || t.visibility() != Visibility::Visible {
                continue;
            }
            samples.push(t.raw_sample());
        }
    }
    if samples.is_empty() {
        return (centre.raw_sample(), vis);
    }
    (LightSample::average(&samples), vis)
}

/// Smoothed *tile-centre* light used by the per-block baker: the
/// tile's own sample averaged with its 4 horizontal neighbours under
/// the same skip rules (occluding / hidden / visibility-mismatched
/// neighbours never contribute). Vertical neighbours are not averaged
/// — z-levels are separate floors; mixing them would leak light
/// through ceilings. Note this centre form *can* soften a wall tile's
/// own colour toward a neighbouring room; that is precisely why the
/// per-vertex face path above is the precise one, and the doc in
/// `bake.rs` says so.
pub fn smoothed_tile_sample<S: LightSampler + ?Sized>(
    s: &S,
    tile: [i32; 3],
) -> (LightSample, Visibility) {
    let centre = s.tile_light(tile[0], tile[1], tile[2]);
    let vis = centre.visibility();
    if vis != Visibility::Visible {
        return (air_side_sample(s, tile), vis);
    }
    let mut samples = vec![centre.raw_sample()];
    for d in [[1, 0], [-1, 0], [0, 1], [0, -1]] {
        let p = [tile[0] + d[0], tile[1] + d[1], tile[2]];
        let t = s.tile_light(p[0], p[1], p[2]);
        if t.occluding || t.visibility() != Visibility::Visible {
            continue;
        }
        samples.push(t.raw_sample());
    }
    (LightSample::average(&samples), vis)
}
