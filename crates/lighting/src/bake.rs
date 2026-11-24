//! Light baking: apply the mapping / visibility / smoothing /
//! environment pipeline to mesher data.
//!
//! Two bakers, two precision levels — the distinction matters and is
//! deliberate:
//!
//! - [`bake_chunk_map`] (per-block): produces a new `ChunkMap` whose
//!   block colors carry light. `BLANK` tiles' blocks become air (not
//!   meshed — gate 7.2), `MEMORIZED` tiles are desaturated + dimmed,
//!   visible tiles are multiplied by their *smoothed tile-centre*
//!   sample. A `Block` has a single color, so a wall block between a
//!   lit and a dark room gets *one* of its tile's centre values on
//!   both faces — this baker is the fast/compatible path (it feeds
//!   the existing mesher unchanged), not the leak-proof one.
//! - [`bake_pass_meshes`] (per-vertex): rewrites the vertex colors
//!   of already-meshed `PassMeshes`, sampling each vertex through
//!   the air-side scheme in `smoothing.rs`. This is the precise path:
//!   the same wall block renders bright on its lit side and dark on
//!   its dark side, and gradients interpolate across chunk borders
//!   with no seam. Renderers that can afford a post-mesh pass should
//!   use it; the parent wiring notes in `docs/phase7-lighting.md`
//!   say where each hooks in.
//!
//! Both bakers are pure functions over `(world, sampler, env)` —
//! same inputs, same outputs, no hidden state.

use mesher::chunk::{Chunk, ChunkMap};
use mesher::mesh::{PassMeshes, FLAT_LAYER};
use mesher::scale::WorldScale;

use crate::atmosphere::Environment;
use crate::grid::LightSampler;
use crate::mapping::{apply_visibility_rgb, LightSample, Visibility, MEMORIZED_BRIGHTNESS};
use crate::smoothing::{smoothed_tile_sample, vertex_light_at_grid_corner, FACE_DIRS};

/// Reference composition for one vertex/face color (used by both
/// bakers and pinned by the gate tests):
/// - `Hidden` -> black
/// - `Memorized` -> desaturate + dim the albedo, environment *tint*
///   still applies but its brightness does not double-dim memory
///   (memory is already dim by definition; the sample passed for a
///   memorized tile is the presented sample with intensity 1.0)
/// - `Visible` -> albedo × environment(sample)
pub fn shade_color(
    albedo: [f32; 3],
    sample: LightSample,
    vis: Visibility,
    env: Environment,
) -> [f32; 3] {
    match vis {
        Visibility::Hidden => [0.0, 0.0, 0.0],
        Visibility::Memorized => {
            let base = apply_visibility_rgb(albedo, vis);
            // Apply only the environment tint (not brightness) so a
            // remembered street at noon and at midnight differs in
            // hue, not in a second brightness hit.
            let tint = [
                env.day_night.tint[0] * env.weather.tint[0],
                env.day_night.tint[1] * env.weather.tint[1],
                env.day_night.tint[2] * env.weather.tint[2],
            ];
            [
                (base[0] * tint[0]).clamp(0.0, 1.0),
                (base[1] * tint[1]).clamp(0.0, 1.0),
                (base[2] * tint[2]).clamp(0.0, 1.0),
            ]
        }
        Visibility::Visible => env.apply(sample).apply_rgb(albedo),
    }
}

fn shade_rgba(
    albedo: [f32; 4],
    sample: LightSample,
    vis: Visibility,
    env: Environment,
) -> [f32; 4] {
    let rgb = shade_color([albedo[0], albedo[1], albedo[2]], sample, vis, env);
    [rgb[0], rgb[1], rgb[2], albedo[3]]
}

/// The pure light multiplier for one vertex — [`shade_color`]
/// with the albedo step factored out (the separation
/// `mapping::presented_sample` documents): the factor the fused
/// composition multiplies the albedo by, per channel. The
/// raster shader's textured path needs exactly this as its
/// emitter ceiling: the texel supplies the albedo, so the
/// vertex colour must carry the multiplier alone — carrying
/// albedo x bake instead would apply both twice.
fn light_multiplier(sample: LightSample, vis: Visibility, env: Environment) -> [f32; 3] {
    match vis {
        Visibility::Hidden => [0.0, 0.0, 0.0],
        Visibility::Memorized => {
            // Environment tint x the MEMORIZED dim, mirroring
            // shade_color's Memorized arm. That arm's other
            // step — desaturation — mixes channels and is not
            // representable in a per-channel multiplier, so a
            // textured memorized tile's emitter cap carries the
            // brightness but not the desaturation.
            let tint = [
                env.day_night.tint[0] * env.weather.tint[0],
                env.day_night.tint[1] * env.weather.tint[1],
                env.day_night.tint[2] * env.weather.tint[2],
            ];
            [
                tint[0] * MEMORIZED_BRIGHTNESS,
                tint[1] * MEMORIZED_BRIGHTNESS,
                tint[2] * MEMORIZED_BRIGHTNESS,
            ]
        }
        Visibility::Visible => env.apply(sample).multiplier(),
    }
}

/// Per-block bake. `grid_z_of_level0` maps chunk level 0 onto the
/// sampler's z coordinate (0 when the grid was built in chunk-level
/// space, the snapshot's minimum game z when built in game space).
pub fn bake_chunk_map_with_offset<S: LightSampler + ?Sized>(
    map: &ChunkMap,
    sampler: &S,
    env: Environment,
    grid_z_of_level0: i32,
) -> ChunkMap {
    let scale = map.scale;
    let per = scale.blocks_per_z_level.max(1) as i32;
    let mut out = ChunkMap::new(scale);
    for (coord, chunk) in &map.chunks {
        let origin = chunk.origin();
        let mut baked = Chunk::new(*coord, chunk.levels, scale);
        for z in 0..chunk.grid.size[2] as i32 {
            let level = z / per;
            let gz = grid_z_of_level0 + level;
            for y in 0..chunk.grid.size[1] as i32 {
                for x in 0..chunk.grid.size[0] as i32 {
                    let b = chunk.grid.get(x, y, z);
                    if b.is_air() {
                        continue;
                    }
                    let tile = [origin[0] + x, origin[1] + y, gz];
                    let vis = sampler.visibility_at(tile[0], tile[1], tile[2]);
                    if vis == Visibility::Hidden {
                        continue; // BLANK: not meshed (stays air)
                    }
                    let (sample, _) = smoothed_tile_sample(sampler, tile);
                    let mut nb = b;
                    nb.color = shade_rgba(b.color, sample, vis, env);
                    baked.grid.set(x as usize, y as usize, z as usize, nb);
                }
            }
        }
        baked.dirty = true;
        out.chunks.insert(*coord, baked);
    }
    out
}

/// Per-block bake in chunk-level z space (grid z == chunk level).
pub fn bake_chunk_map<S: LightSampler + ?Sized>(
    map: &ChunkMap,
    sampler: &S,
    env: Environment,
) -> ChunkMap {
    bake_chunk_map_with_offset(map, sampler, env, 0)
}

/// Keep only the given chunk levels' blocks (a data-level cutaway,
/// the same operation `engine::world::filtered_map` performs for
/// the renderers). Used by the isometric dump to show the lit
/// interior instead of the dark roof slab.
pub fn filter_levels(map: &ChunkMap, keep_levels: &[i32]) -> ChunkMap {
    let scale = map.scale;
    let per = scale.blocks_per_z_level.max(1) as i32;
    let mut out = ChunkMap::new(scale);
    for (coord, chunk) in &map.chunks {
        let mut fc = Chunk::new(*coord, chunk.levels, scale);
        for z in 0..chunk.grid.size[2] as i32 {
            if !keep_levels.contains(&(z / per)) {
                continue;
            }
            for y in 0..chunk.grid.size[1] as i32 {
                for x in 0..chunk.grid.size[0] as i32 {
                    let b = chunk.grid.get(x, y, z);
                    if !b.is_air() {
                        fc.grid.set(x as usize, y as usize, z as usize, b);
                    }
                }
            }
        }
        fc.dirty = false;
        out.chunks.insert(*coord, fc);
    }
    out
}

/// Per-vertex bake over combined meshes (the precise, leak-proof
/// path). `meshes` must have been built from *unbaked* albedos (flat
/// block colors) at `scale`, so this composes with either mesher and
/// with the engine's `mesh_world` output directly.
///
/// How a vertex finds its light: a sample point half a block behind
/// the face (`position - normal * 0.5`, in grid space) identifies
/// the *solid tile* the face belongs to. Side faces then sample the
/// air tile in front of them (solid tile + horizontal normal) with
/// horizontal vertex averaging; top/bottom faces sample the solid
/// tile itself (smoothing rule 3 — a floor slab's airspace is its
/// own tile, and a roof's top face takes the roof tile's game light
/// rather than the BLANK void above the stored world).
pub fn bake_pass_meshes<S: LightSampler + ?Sized>(
    meshes: &mut PassMeshes,
    sampler: &S,
    env: Environment,
    scale: WorldScale,
    grid_z_of_level0: i32,
) {
    let bs = scale.block_size;
    let per = scale.blocks_per_z_level.max(1) as i32;
    for mesh in meshes.meshes.iter_mut() {
        // Defensive: `layers` is documented parallel to the
        // vertex arrays; if a mesh's layers don't line up with
        // its vertices, treat every vertex as flat rather than
        // guess which ones are textured.
        let layers_ok = mesh.layers.len() == mesh.positions.len();
        for i in 0..mesh.positions.len() {
            let p = mesh.positions[i];
            let n = mesh.normals[i];
            // World -> grid (inverse of mesher's (x, z, -y) rotation).
            let gp = [p[0] / bs, -p[2] / bs, p[1] / bs];
            let gn = [n[0], -n[2], n[1]];
            let di = FACE_DIRS
                .iter()
                .position(|d| d[0] as f32 == gn[0] && d[1] as f32 == gn[1] && d[2] as f32 == gn[2])
                .unwrap_or(4);
            let d = FACE_DIRS[di];
            let axis = (0..3).find(|&a| d[a] != 0).unwrap();
            // Solid block / tile behind this face at this vertex.
            let sp = [
                gp[0] - gn[0] * 0.5,
                gp[1] - gn[1] * 0.5,
                gp[2] - gn[2] * 0.5,
            ];
            let solid_tile = [
                sp[0].floor() as i32,
                sp[1].floor() as i32,
                (sp[2].floor() as i32).div_euclid(per) + grid_z_of_level0,
            ];
            let (sample, vis) = if axis == 2 {
                // Top/bottom face: own tile, horizontal averaging.
                let corner_xy = [gp[0].round() as i32, gp[1].round() as i32];
                crate::smoothing::top_face_vertex_light(sampler, solid_tile, corner_xy)
            } else {
                // Side face: air tile in front of the face; the
                // grid corner (tile space) carries the horizontal
                // in-plane coordinate for averaging.
                let air_tile = [solid_tile[0] + d[0], solid_tile[1] + d[1], solid_tile[2]];
                let mut corner_tile = air_tile;
                for a in 0..3 {
                    if a != axis && a != 2 {
                        corner_tile[a] = gp[a].round() as i32;
                    }
                }
                vertex_light_at_grid_corner(sampler, air_tile, di, corner_tile)
            };
            let albedo = mesh.colors[i];
            if layers_ok && mesh.layers[i] != FLAT_LAYER {
                // Textured vertex (ordinary, macro and blend
                // layer words alike): store the pure light
                // multiplier, NOT shade_rgba's fused
                // albedo x bake — the shader samples the albedo
                // from the texture array and needs the bake
                // alone, both as the emitter ceiling and to keep
                // the albedo applied exactly once. Alpha is
                // preserved exactly as shade_rgba preserves it.
                let m = light_multiplier(sample, vis, env);
                mesh.colors[i] = [m[0], m[1], m[2], albedo[3]];
            } else {
                mesh.colors[i] = shade_rgba(albedo, sample, vis, env);
            }
        }
    }
}
