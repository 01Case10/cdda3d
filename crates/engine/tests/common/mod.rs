//! Shared capture classifier for the Stage F2 gates (pond +
//! blend), reworked 2026-11-23 for Phase 10 §5.3.
//!
//! The old classifiers were channel-ordering rules
//! (`dirt_like`: r > g > b with (r − b) ≥ 20; `water_like`:
//! b > g > r; `grass_like`: g > r && g > b) whose doc comment
//! claimed "multiplicative face shading preserves channel
//! ordering". That invariant holds only for *scalar* light.
//! §5.3's hemisphere ambient is coloured — at neutral the
//! zenith is (0.30, 0.48, 0.78), blue weighted 2.6× over red —
//! so a low-saturation albedo like dirt (120, 95, 60), whose
//! r−b spread is small, has its ordering flipped by the
//! ambient alone: the RT pond gate's dirt count went
//! 982 → 0 while the renderer was shading correctly.
//!
//! The replacement classifies in *chromaticity* space
//! (rgb / (r+g+b)), which is invariant to the scalar parts of
//! shading (face light, AO, lit factor), against reference
//! centroids computed by pushing each gate's own albedos
//! through the frozen neutral illuminants — the classifier
//! derives what each class should look like under the actual
//! lighting instead of hard-coding what it looked like under
//! the old lighting.
//!
//! Measured derivation (lavapipe captures at
//! `SkyUniforms::neutral()`, post-sun-direction-fix trees,
//! 2026-11-23; regions labelled by shore-vs-plain capture
//! diff for the bank and by pond bounding box for the
//! interior):
//!
//! * RT top-face illuminants at neutral: ambient-only =
//!   zenith (0.30, 0.48, 0.78); sun-lit = zenith + white ×
//!   ndl, ndl = 0.848 for the legacy grid sun dir
//!   (0.3, −0.4, 0.8) — i.e. (1.148, 1.328, 1.628).
//! * RT measured clusters vs computed lit centroids: grass
//!   tops (0.261, 0.491, 0.248) vs (0.268, 0.487, 0.244);
//!   pond water (0.103, 0.268, 0.629) vs (0.085, 0.264,
//!   0.651); bank dirt main colour (75, 70, 58) →
//!   (0.369, 0.345, 0.286) vs (0.381, 0.349, 0.270).
//! * RT dirt gets NO ambient centroid: the computed ambient
//!   dirt centroid (0.280, 0.355, 0.364) sits 0.033 from the
//!   measured sky-adjacent colours (64, 74, 86) / (71, 82,
//!   95) → chroma (0.286, 0.330, 0.384), of which the plain
//!   capture carries 624 px — they would false-positive the
//!   no-bank control. The ambient-only bank-dirt population
//!   measured only ~33 px (the bank tops are sky-exposed),
//!   so Dirt = the lit centroid alone. Water and grass
//!   ambient centroids are ≥ 0.24 from the sky gradient's
//!   chromaticity path and are kept.
//! * Raster at neutral is scalar by construction (§5.2's
//!   calibrated GI block: hemisphere 0.45 grey + sun 0.55
//!   white), so its computed centroids are the albedo
//!   chromaticities themselves. Two measured exceptions,
//!   both from the shore-variant vertex tint: bank dirt's
//!   main cluster sits at (0.396, 0.332, 0.272) in the 10.3
//!   raster build and (0.375, 0.331, 0.295) in the legacy
//!   build — 0.07–0.10 off the scalar centroid — and pond
//!   water at (0.183, 0.295, 0.522) / (0.204, 0.298, 0.498)
//!   vs scalar (0.111, 0.296, 0.593). Each class therefore
//!   carries the computed scalar centroid plus one measured
//!   centroid midway between the two builds.
//! * Acceptance radius 0.06 in chromaticity, chroma floor
//!   0.04 (distance from the grey point (⅓, ⅓, ⅓); sub-floor
//!   pixels — deep shadow, near-grey — classify as Other
//!   rather than being forced into a class). Recovered
//!   counts at these constants: RT dirt 865 (pre-GI rule:
//!   982; broken rule: 0), raster dirt 844–879 (old rule:
//!   875–930), plain-capture dirt ≤ 1 in every capture.

// Each gate crate includes this whole module but uses only
// its own subset of the entry points.
#![allow(dead_code)]

use engine::capture::RendererKind;
use image::Rgba;
use lighting::{SkyUniforms, LEGACY_GROUND, LEGACY_SUN_DIR_GRID, LEGACY_ZENITH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    Dirt,
    Water,
    Grass,
    Other,
}

/// One reference point in chromaticity space.
pub struct Centroid {
    pub class: Class,
    pub chroma: [f32; 3],
}

/// Max chromaticity distance from the nearest centroid for a
/// pixel to take that centroid's class.
pub const MAX_DIST: f32 = 0.06;
/// Min chromaticity distance from grey for a pixel to be
/// classifiable at all.
pub const CHROMA_FLOOR: f32 = 0.04;

pub fn chromaticity(rgb: [f32; 3]) -> [f32; 3] {
    let s = rgb[0] + rgb[1] + rgb[2];
    if s <= 0.0 {
        return [0.0; 3];
    }
    [rgb[0] / s, rgb[1] / s, rgb[2] / s]
}

fn shaded_chroma(albedo: [u8; 3], illuminant: [f32; 3]) -> [f32; 3] {
    chromaticity([
        albedo[0] as f32 * illuminant[0],
        albedo[1] as f32 * illuminant[1],
        albedo[2] as f32 * illuminant[2],
    ])
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// Nearest-centroid classification with rejection: a pixel
/// farther than [`MAX_DIST`] from every centroid, or closer
/// than [`CHROMA_FLOOR`] to grey, is [`Class::Other`].
pub fn classify(px: &Rgba<u8>, centroids: &[Centroid]) -> Class {
    let c = chromaticity([px.0[0] as f32, px.0[1] as f32, px.0[2] as f32]);
    if dist(c, [1.0 / 3.0; 3]) < CHROMA_FLOOR {
        return Class::Other;
    }
    let mut best = Class::Other;
    let mut best_d = f32::MAX;
    for cen in centroids {
        let d = dist(c, cen.chroma);
        if d < best_d {
            best_d = d;
            best = cen.class;
        }
    }
    if best_d <= MAX_DIST {
        best
    } else {
        Class::Other
    }
}

pub fn count_class(img: &image::RgbaImage, centroids: &[Centroid], class: Class) -> usize {
    img.pixels().filter(|p| classify(p, centroids) == class).count()
}

/// The RT renderer's neutral top-face illuminants, from the
/// frozen `SkyUniforms::neutral()` block: ambient-only is the
/// zenith; sun-lit adds the white sun at the legacy grid
/// direction's up-ndl. (Side-face hemisphere mixes sit
/// within ~0.01 chromaticity of the top ambient for these
/// albedos, so no separate side centroids are carried.)
fn rt_illuminants() -> ([f32; 3], [f32; 3]) {
    let neutral = SkyUniforms::neutral();
    let scale = neutral.ambient_scale();
    let zenith = [
        neutral.zenith3()[0] * scale,
        neutral.zenith3()[1] * scale,
        neutral.zenith3()[2] * scale,
    ];
    debug_assert_eq!(zenith, LEGACY_ZENITH);
    debug_assert_eq!(neutral.ground3(), LEGACY_GROUND);
    let sun = LEGACY_SUN_DIR_GRID;
    let len = (sun[0] * sun[0] + sun[1] * sun[1] + sun[2] * sun[2]).sqrt();
    let ndl_up = sun[2] / len;
    let sun_col = neutral.sun_color;
    let lit = [
        zenith[0] + sun_col[0] * ndl_up,
        zenith[1] + sun_col[1] * ndl_up,
        zenith[2] + sun_col[2] * ndl_up,
    ];
    (zenith, lit)
}

/// Centroids for the pond gate (dirt bank / water interior /
/// grass surround) for one renderer's captures.
pub fn pond_centroids(
    renderer: RendererKind,
    dirt: [u8; 3],
    water: [u8; 3],
    grass: [u8; 3],
) -> Vec<Centroid> {
    let mut out = Vec::new();
    let mut push = |class: Class, chroma: [f32; 3]| out.push(Centroid { class, chroma });
    match renderer {
        RendererKind::Rt => {
            let (amb, lit) = rt_illuminants();
            // Dirt: lit only — see the module doc for the
            // measured sky degeneracy of the ambient centroid.
            push(Class::Dirt, shaded_chroma(dirt, lit));
            push(Class::Water, shaded_chroma(water, lit));
            push(Class::Water, shaded_chroma(water, amb));
            push(Class::Grass, shaded_chroma(grass, lit));
            push(Class::Grass, shaded_chroma(grass, amb));
        }
        RendererKind::Raster => {
            // Scalar light: computed centroid = albedo
            // chromaticity; plus the measured shore-tint
            // centroids (module doc).
            push(Class::Dirt, shaded_chroma(dirt, [1.0; 3]));
            push(Class::Dirt, [0.390, 0.330, 0.280]);
            push(Class::Water, shaded_chroma(water, [1.0; 3]));
            push(Class::Water, [0.190, 0.300, 0.510]);
            push(Class::Grass, shaded_chroma(grass, [1.0; 3]));
        }
    }
    out
}

/// Centroids for the blend gate (grass | dirt ground halves,
/// no water) for one renderer's captures. Same construction
/// as the pond gate's dirt/grass sets.
pub fn blend_centroids(renderer: RendererKind, dirt: [u8; 3], grass: [u8; 3]) -> Vec<Centroid> {
    pond_centroids(renderer, dirt, [0; 3], grass)
        .into_iter()
        .filter(|c| c.class != Class::Water)
        .collect()
}
