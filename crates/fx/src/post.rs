//! HD-2D post stack (Phase 2b): bloom, tilt-shift depth of field,
//! warm grade + vignette, optional chromatic offset.
//!
//! Renderer-agnostic by construction: the stack consumes a finished
//! color frame plus a depth frame and produces a graded frame. The
//! identical stack applies to the Phase 2 raster renderer and the
//! Phase 2c ray tracer. It applies to the WORLD ONLY — the UI is
//! composited after the stack and is never graded (gate 2b.7).
//!
//! This module is the CPU reference implementation (used by the gate
//! tests and as the spec for the WGSL pass in `render.rs`). Each pass
//! is independently toggleable; with everything off the output is
//! pixel-identical to the input (gate 2b.6).

/// Toggleable post configuration. The same struct drives the CPU
/// reference and the GPU pass uniforms.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PostConfig {
    pub bloom: bool,
    pub dof: bool,
    pub grade: bool,
    pub chromatic: bool,
    /// Luminance above this feeds bloom.
    pub bloom_threshold: f32,
    pub bloom_strength: f32,
    /// Depth (view-space distance) of the follow target: the DoF
    /// sharp band sits here.
    pub focal_depth: f32,
    /// How fast blur grows away from the focal depth.
    pub dof_strength: f32,
}

impl Default for PostConfig {
    fn default() -> Self {
        Self::all_on()
    }
}

impl PostConfig {
    pub fn all_off() -> Self {
        PostConfig {
            bloom: false,
            dof: false,
            grade: false,
            chromatic: false,
            bloom_threshold: 0.75,
            bloom_strength: 0.6,
            focal_depth: 18.0,
            dof_strength: 1.0,
        }
    }
    pub fn all_on() -> Self {
        PostConfig {
            bloom: true,
            dof: true,
            grade: true,
            chromatic: true,
            ..PostConfig::all_off()
        }
    }
    pub fn any_on(&self) -> bool {
        self.bloom || self.dof || self.grade || self.chromatic
    }
}

/// A finished world frame: RGBA pixels (0..=1 floats) + per-pixel
/// view depth. Row-major, `width * height` entries each.
#[derive(Debug, Clone)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub color: Vec<[f32; 4]>,
    pub depth: Vec<f32>,
}

impl Frame {
    pub fn new(width: u32, height: u32, color: [f32; 4], depth: f32) -> Self {
        let n = (width * height) as usize;
        Frame {
            width,
            height,
            color: vec![color; n],
            depth: vec![depth; n],
        }
    }
    pub fn idx(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }
}

fn luminance(c: [f32; 4]) -> f32 {
    c[0] * 0.299 + c[1] * 0.587 + c[2] * 0.114
}

/// Separable-ish box blur (single square kernel pass; the reference
/// favors clarity over speed — it runs in tests, not per frame).
fn box_blur(color: &[[f32; 4]], w: u32, h: u32, radius: u32) -> Vec<[f32; 4]> {
    if radius == 0 {
        return color.to_vec();
    }
    let r = radius as i32;
    let mut out = vec![[0.0f32; 4]; color.len()];
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let mut acc = [0.0f32; 4];
            let mut n = 0.0f32;
            for dy in -r..=r {
                for dx in -r..=r {
                    let sx = (x + dx).clamp(0, w as i32 - 1);
                    let sy = (y + dy).clamp(0, h as i32 - 1);
                    let p = color[(sy * w as i32 + sx) as usize];
                    for c in 0..4 {
                        acc[c] += p[c];
                    }
                    n += 1.0;
                }
            }
            let o = &mut out[(y * w as i32 + x) as usize];
            for c in 0..4 {
                o[c] = acc[c] / n;
            }
        }
    }
    out
}

/// Bloom: additive blurred bright-pass. Adds light only — it cannot
/// shift geometry (gate 2b.6).
pub fn bloom_pass(color: &[[f32; 4]], w: u32, h: u32, cfg: &PostConfig) -> Vec<[f32; 4]> {
    let bright: Vec<[f32; 4]> = color
        .iter()
        .map(|c| {
            let over = (luminance(*c) - cfg.bloom_threshold).max(0.0);
            let scale = if luminance(*c) > 0.0 {
                over / luminance(*c)
            } else {
                0.0
            };
            [c[0] * scale, c[1] * scale, c[2] * scale, 0.0]
        })
        .collect();
    let blurred = box_blur(&bright, w, h, 2);
    color
        .iter()
        .zip(blurred.iter())
        .map(|(c, b)| {
            [
                c[0] + b[0] * cfg.bloom_strength,
                c[1] + b[1] * cfg.bloom_strength,
                c[2] + b[2] * cfg.bloom_strength,
                c[3],
            ]
        })
        .collect()
}

/// Tilt-shift DoF: pixels at the focal depth stay exactly as they
/// are; blur grows with |depth − focal| (and slightly with vertical
/// distance from the focal row — the tilt-shift band).
pub fn dof_pass(frame: &Frame, color: &[[f32; 4]], cfg: &PostConfig) -> Vec<[f32; 4]> {
    let (w, h) = (frame.width, frame.height);
    let blurred = box_blur(color, w, h, 3);
    let center_row = h as f32 / 2.0;
    let mut out = color.to_vec();
    for y in 0..h {
        for x in 0..w {
            let i = frame.idx(x, y);
            let depth_err = (frame.depth[i] - cfg.focal_depth).abs();
            let tilt = 1.0 + 0.6 * ((y as f32 - center_row).abs() / center_row.max(1.0));
            let amt = ((depth_err - 1.5) * 0.25 * cfg.dof_strength * tilt).clamp(0.0, 1.0);
            if amt > 0.0 {
                for c in 0..3 {
                    out[i][c] = color[i][c] * (1.0 - amt) + blurred[i][c] * amt;
                }
            }
        }
    }
    out
}

/// Chromatic offset: red/blue channels sampled ±1 px horizontally,
/// scaled by distance from the image center.
pub fn chromatic_pass(color: &[[f32; 4]], w: u32, h: u32) -> Vec<[f32; 4]> {
    let mut out = color.to_vec();
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            let fx = (x as f32 / w.max(1) as f32 - 0.5).abs() * 2.0;
            if fx < 0.2 {
                continue; // center stays clean
            }
            let xr = ((x as i32 + 1).min(w as i32 - 1)) as u32;
            let xb = ((x as i32 - 1).max(0)) as u32;
            out[i][0] = color[(y * w + xr) as usize][0];
            out[i][2] = color[(y * w + xb) as usize][2];
        }
    }
    out
}

/// Warm grade + vignette: the HD-2D finish. Multiplies color toward
/// warm and darkens corners; alpha untouched.
pub fn grade_pass(color: &[[f32; 4]], w: u32, h: u32) -> Vec<[f32; 4]> {
    let mut out = color.to_vec();
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            let nx = x as f32 / w.max(1) as f32 * 2.0 - 1.0;
            let ny = y as f32 / h.max(1) as f32 * 2.0 - 1.0;
            let vig = 1.0 - 0.32 * ((nx * nx + ny * ny) / 2.0).min(1.0);
            out[i][0] = (color[i][0] * 1.07 * vig).clamp(0.0, 1.0);
            out[i][1] = (color[i][1] * 1.01 * vig).clamp(0.0, 1.0);
            out[i][2] = (color[i][2] * 0.93 * vig).clamp(0.0, 1.0);
        }
    }
    out
}

/// Apply the enabled passes in stack order: bloom → DoF → chromatic
/// → grade. All-off returns the input pixels unchanged, exactly
/// (gate 2b.6 identity).
pub fn apply_post_cpu(frame: &Frame, cfg: &PostConfig) -> Vec<[f32; 4]> {
    if !cfg.any_on() {
        return frame.color.clone();
    }
    let (w, h) = (frame.width, frame.height);
    let mut color = frame.color.clone();
    if cfg.bloom {
        color = bloom_pass(&color, w, h, cfg);
    }
    if cfg.dof {
        color = dof_pass(frame, &color, cfg);
    }
    if cfg.chromatic {
        color = chromatic_pass(&color, w, h);
    }
    if cfg.grade {
        color = grade_pass(&color, w, h);
    }
    color
}

/// Composite a UI frame OVER a (possibly graded) world frame. This
/// is the only place UI pixels enter the pipeline, and it happens
/// strictly after the post stack — UI pixels with alpha 1 come out
/// bit-exact (gate 2b.7).
pub fn composite_ui(world: &[[f32; 4]], ui: &[[f32; 4]]) -> Vec<[f32; 4]> {
    assert_eq!(world.len(), ui.len());
    world
        .iter()
        .zip(ui.iter())
        .map(|(w, u)| {
            let a = u[3].clamp(0.0, 1.0);
            [
                u[0] * a + w[0] * (1.0 - a),
                u[1] * a + w[1] * (1.0 - a),
                u[2] * a + w[2] * (1.0 - a),
                1.0,
            ]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_off_is_identity() {
        let mut f = Frame::new(8, 8, [0.2, 0.4, 0.6, 1.0], 10.0);
        f.color[3] = [0.9, 0.1, 0.3, 1.0];
        let out = apply_post_cpu(&f, &PostConfig::all_off());
        assert_eq!(out, f.color);
    }
}
