//! Phase 2e: ray-cone texture LOD (docs/phase2e-mipmaps.md §2) —
//! the CPU reference for the WGSL `cone_lod` in [`crate::render`],
//! plus the CPU mip-chain generation used by
//! `TextureSet::from_spec`.
//!
//! The renderer is a compute shader: no implicit derivatives, so
//! LOD is explicit. A primary ray carries a cone whose full-angle
//! spread is fixed by the camera (`spread_angle`); at a hit at
//! distance `t` the cone footprint is `w_hit = spread * t` world
//! units wide. Every voxel face maps the full texture over exactly
//! 1x1 world units, so the per-face term collapses to the constant
//! `Δ = 0.5 * log2(texW * texH)` and
//! `lod = Δ + log2(w_hit) - log2(|N·V|)`, clamped to the chain.
//!
//! Mip generation mirrors the voxelizer's Phase 5 §8 rules
//! (`crates/voxelizer/src/faces.rs`): premultiplied alpha-aware 2x2
//! averaging (fully transparent texels contribute no color, so
//! cutout edges don't bleed), edge texels clamp, and the result is
//! un-premultiplied on output. Averaging happens in the stored
//! (display) space, consistently with how level 0 is sampled.
//! On top of that, each generated level's alpha coverage is
//! rescaled so its mean alpha tracks level 0's — without it,
//! cutout textures thin out at coarse mips.

/// Full-angle ray-cone spread (radians per unit distance) for a
/// camera with the given `tan(fovY / 2)` rendering into an image
/// `image_height` pixels tall: `atan(2 * tan(fovY/2) / height)`.
pub fn spread_angle(tan_half_fov: f32, image_height: u32) -> f32 {
    if image_height == 0 {
        return 0.0;
    }
    (2.0 * tan_half_fov / image_height as f32).atan()
}

/// Stage A tunables (procedural texture pipeline rework, draft
/// §4.11/§6.1): a constant LOD bias and a weight on the slant term,
/// added because the unweighted `-log2(|N·V|)` term over-blurred
/// surfaces seen at a tilt. The starting values are the draft's:
/// bias −0.5 (half a mip sharper everywhere) and weight 0.5 (half
/// the slant penalty). These are the values the WGSL `cone_lod`
/// receives through the uniforms; tune them here and in the
/// shader together — `mip_gate` cross-checks CPU vs GPU.
pub const LOD_BIAS: f32 = -0.5;
pub const NDOTV_WEIGHT: f32 = 0.5;

/// Ray-cone LOD for a hit on a voxel face textured with a
/// `tex_w` x `tex_h` layer:
/// `Δ + log2(spread * t) - NDOTV_WEIGHT * log2(|N·V|) + LOD_BIAS`
/// clamped to `[0, max_mip]`.
///
/// Hygiene (docs/phase2e-mipmaps.md §2): `w_hit` and `|N·V|` are
/// epsilon-clamped to `>= 1e-4` before `log2` (grazing faces are
/// where the slant term explodes; the `max_mip` clamp keeps them
/// finite), and a NaN result collapses to 0.
pub fn ray_cone_lod(tex_w: u32, tex_h: u32, spread: f32, t: f32, n_dot_v: f32, max_mip: f32) -> f32 {
    let delta = 0.5 * ((tex_w.max(1) * tex_h.max(1)) as f32).log2();
    let w_hit = (spread * t).max(1e-4);
    let cos = n_dot_v.abs().max(1e-4);
    let lod = delta + w_hit.log2() - NDOTV_WEIGHT * cos.log2() + LOD_BIAS;
    if !lod.is_finite() {
        return 0.0;
    }
    lod.clamp(0.0, max_mip.max(0.0))
}

/// One 2x downsample step of a square RGBA layer (`size` x `size`,
/// row-major, 4 bytes per texel), premultiplied alpha-aware with
/// edge clamping — the voxelizer `faces::downsample2x` rules on raw
/// byte buffers. Output size is `(size / 2).max(1)` squared.
pub fn downsample2x(layer: &[u8], size: u32) -> Vec<u8> {
    let size = size.max(1);
    debug_assert_eq!(layer.len(), (size * size * 4) as usize);
    let out_size = (size / 2).max(1);
    let px = |x: u32, y: u32| -> [u8; 4] {
        let i = ((y * size + x) * 4) as usize;
        [layer[i], layer[i + 1], layer[i + 2], layer[i + 3]]
    };
    let mut out = Vec::with_capacity((out_size * out_size * 4) as usize);
    for y in 0..out_size {
        for x in 0..out_size {
            let mut sum = [0f32; 4];
            let mut count = 0f32;
            for dy in 0..2u32 {
                for dx in 0..2u32 {
                    let sx = (x * 2 + dx).min(size - 1);
                    let sy = (y * 2 + dy).min(size - 1);
                    let p = px(sx, sy);
                    let a = p[3] as f32 / 255.0;
                    sum[0] += p[0] as f32 * a;
                    sum[1] += p[1] as f32 * a;
                    sum[2] += p[2] as f32 * a;
                    sum[3] += a;
                    count += 1.0;
                }
            }
            let mean_a = sum[3] / count;
            let mut out_px = [0u8; 4];
            for c in 0..3 {
                let premul_mean = sum[c] / count;
                out_px[c] = if mean_a > 0.001 {
                    (premul_mean / mean_a).round().clamp(0.0, 255.0) as u8
                } else {
                    0
                };
            }
            out_px[3] = (mean_a * 255.0).round() as u8;
            out.extend_from_slice(&out_px);
        }
    }
    out
}

fn mean_alpha(layer: &[u8]) -> f32 {
    if layer.is_empty() {
        return 0.0;
    }
    let n = (layer.len() / 4) as f32;
    layer
        .iter()
        .skip(3)
        .step_by(4)
        .map(|&a| a as f32)
        .sum::<f32>()
        / n
}

/// Rescale a generated level so its mean alpha equals
/// `target_mean_alpha` (level 0's): premultiplied color is scaled
/// together with alpha, which leaves the un-premultiplied RGB
/// unchanged and only moves coverage. No-op when either mean is 0
/// or the level is already on target.
fn rescale_alpha_coverage(level: &mut [u8], target_mean_alpha: f32) {
    let current = mean_alpha(level);
    if current <= 0.0 || target_mean_alpha <= 0.0 {
        return;
    }
    let factor = target_mean_alpha / current;
    if (factor - 1.0).abs() < 1e-6 {
        return;
    }
    for px in level.chunks_exact_mut(4) {
        // Scale premultiplied color with alpha, then un-premultiply:
        // RGB survives unchanged (up to the alpha clamp at 255);
        // a texel whose alpha is scaled to 0 loses its color, the
        // same convention as the downsample's un-premultiply.
        let a = px[3] as f32;
        let new_a = (a * factor).round().clamp(0.0, 255.0);
        if new_a <= 0.0 {
            px[0] = 0;
            px[1] = 0;
            px[2] = 0;
        }
        px[3] = new_a as u8;
    }
}

/// Full mip chain for a square RGBA layer: level 0 is the input,
/// each following level is `downsample2x` of the previous, down to
/// 1x1. Every generated level's mean alpha is rescaled to track
/// level 0's (alpha-coverage preservation for cutout materials);
/// for opaque layers (level-0 mean alpha 255) the rescale is a
/// no-op.
pub fn mip_chain(layer: &[u8], size: u32) -> Vec<Vec<u8>> {
    let size = size.max(1);
    debug_assert_eq!(layer.len(), (size * size * 4) as usize);
    let target_mean_alpha = mean_alpha(layer);
    let opaque = target_mean_alpha >= 255.0;
    let mut chain = vec![layer.to_vec()];
    let mut cur_size = size;
    while cur_size > 1 {
        let mut next = downsample2x(chain.last().unwrap(), cur_size);
        if !opaque {
            rescale_alpha_coverage(&mut next, target_mean_alpha);
        }
        chain.push(next);
        cur_size = (cur_size / 2).max(1);
    }
    chain
}
