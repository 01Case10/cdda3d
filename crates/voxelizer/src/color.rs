//! Color science for the procedural texture pipeline (Phase 5 rework,
//! Stage B): sRGB <-> linear-light conversion, OKLab (Björn Ottosson)
//! conversion, and palette distance.
//!
//! Everything here is pure and deterministic: fixed constants, no
//! lookup tables built from runtime state, no allocation-dependent
//! iteration order. OKLab is the working space for palette extraction
//! and color-family classification because Euclidean distance in OKLab
//! tracks perceived color difference far better than sRGB distance, so
//! "nearest palette centroid" is meaningful.

/// sRGB electro-optical transfer function: one channel, 0..=1 -> linear.
pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Inverse of [`srgb_to_linear`]: linear 0..=1 -> sRGB 0..=1.
pub fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// 8-bit sRGB triple -> linear-light RGB, channels in 0..=1.
pub fn rgb_u8_to_linear(rgb: [u8; 3]) -> [f32; 3] {
    [
        srgb_to_linear(rgb[0] as f32 / 255.0),
        srgb_to_linear(rgb[1] as f32 / 255.0),
        srgb_to_linear(rgb[2] as f32 / 255.0),
    ]
}

/// Linear-light RGB (0..=1) -> 8-bit sRGB triple (clamped, rounded).
pub fn linear_to_rgb_u8(rgb: [f32; 3]) -> [u8; 3] {
    let mut out = [0u8; 3];
    for (i, c) in rgb.iter().enumerate() {
        out[i] = (linear_to_srgb(c.clamp(0.0, 1.0)) * 255.0).round() as u8;
    }
    out
}

/// A color in OKLab space: `l` lightness (0..=1 for in-gamut colors),
/// `a` green<->red, `b` blue<->yellow.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oklab {
    pub l: f32,
    pub a: f32,
    pub b: f32,
}

impl Oklab {
    pub const BLACK: Oklab = Oklab {
        l: 0.0,
        a: 0.0,
        b: 0.0,
    };

    /// Linear-light sRGB -> OKLab (Ottosson's matrices).
    pub fn from_linear_rgb(rgb: [f32; 3]) -> Oklab {
        let (r, g, b) = (rgb[0], rgb[1], rgb[2]);
        let l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
        let m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
        let s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;
        let (l, m, s) = (l.cbrt(), m.cbrt(), s.cbrt());
        Oklab {
            l: 0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
            a: 1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
            b: 0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
        }
    }

    /// OKLab -> linear-light sRGB (may leave 0..=1 for out-of-gamut
    /// inputs; callers clamp where it matters).
    pub fn to_linear_rgb(self) -> [f32; 3] {
        let l = self.l + 0.3963377774 * self.a + 0.2158037573 * self.b;
        let m = self.l - 0.1055613458 * self.a - 0.0638541728 * self.b;
        let s = self.l - 0.0894841775 * self.a - 1.2914855480 * self.b;
        let (l, m, s) = (l * l * l, m * m * m, s * s * s);
        [
            4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
            -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
            -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
        ]
    }

    pub fn from_srgb_u8(rgb: [u8; 3]) -> Oklab {
        Oklab::from_linear_rgb(rgb_u8_to_linear(rgb))
    }

    pub fn to_srgb_u8(self) -> [u8; 3] {
        linear_to_rgb_u8(self.to_linear_rgb())
    }

    /// Euclidean distance in OKLab space (the palette distance used by
    /// k-means and the classifier's color fallback). Roughly: 0.02 is
    /// a just-noticeable difference, 0.1 is clearly a different shade,
    /// 0.3+ is a different color family.
    pub fn distance(self, other: Oklab) -> f32 {
        let (dl, da, db) = (self.l - other.l, self.a - other.a, self.b - other.b);
        (dl * dl + da * da + db * db).sqrt()
    }

    /// Component-wise weighted mean of a set of colors; `weights` must
    /// be non-empty and parallel to `colors`. Used for palette
    /// centroids. Panics on mismatched/empty input (programmer error).
    pub fn weighted_mean(colors: &[Oklab], weights: &[f32]) -> Oklab {
        assert_eq!(colors.len(), weights.len(), "weighted_mean: length mismatch");
        assert!(!colors.is_empty(), "weighted_mean: empty input");
        let mut total = 0.0f32;
        let mut acc = Oklab::BLACK;
        for (c, w) in colors.iter().zip(weights.iter()) {
            acc.l += c.l * w;
            acc.a += c.a * w;
            acc.b += c.b * w;
            total += w;
        }
        if total > 0.0 {
            acc.l /= total;
            acc.a /= total;
            acc.b /= total;
        }
        acc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32, tol: f32) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn srgb_linear_roundtrip() {
        for v in [0u8, 1, 17, 64, 128, 200, 254, 255] {
            let lin = rgb_u8_to_linear([v, v, v]);
            let back = linear_to_rgb_u8(lin);
            assert_eq!(back, [v, v, v], "gray roundtrip at {v}");
        }
        for rgb in [[255, 0, 0], [12, 200, 99], [1, 2, 3], [250, 128, 5]] {
            let back = linear_to_rgb_u8(rgb_u8_to_linear(rgb));
            for c in 0..3 {
                assert!(
                    (back[c] as i32 - rgb[c] as i32).abs() <= 1,
                    "roundtrip {rgb:?} -> {back:?}"
                );
            }
        }
    }

    #[test]
    fn oklab_known_values() {
        // Published OKLab values (Ottosson) for the sRGB primaries.
        let red = Oklab::from_srgb_u8([255, 0, 0]);
        assert!(close(red.l, 0.62796, 0.001), "red L {}", red.l);
        assert!(close(red.a, 0.22486, 0.001), "red a {}", red.a);
        assert!(close(red.b, 0.12585, 0.001), "red b {}", red.b);
        let green = Oklab::from_srgb_u8([0, 255, 0]);
        assert!(close(green.l, 0.86644, 0.001), "green L {}", green.l);
        assert!(close(green.a, -0.23389, 0.001), "green a {}", green.a);
        assert!(close(green.b, 0.17950, 0.001), "green b {}", green.b);
        let blue = Oklab::from_srgb_u8([0, 0, 255]);
        assert!(close(blue.l, 0.45201, 0.001), "blue L {}", blue.l);
        assert!(close(blue.a, -0.03246, 0.001), "blue a {}", blue.a);
        assert!(close(blue.b, -0.31153, 0.001), "blue b {}", blue.b);
        let white = Oklab::from_srgb_u8([255, 255, 255]);
        assert!(close(white.l, 1.0, 0.001), "white L {}", white.l);
        assert!(white.a.abs() < 0.001 && white.b.abs() < 0.001);
        assert_eq!(Oklab::from_srgb_u8([0, 0, 0]), Oklab::BLACK);
    }

    #[test]
    fn oklab_roundtrip_bound() {
        // sRGB -> OKLab -> sRGB must return within 1 step per channel.
        for rgb in [
            [255, 0, 0],
            [0, 255, 0],
            [0, 0, 255],
            [94, 128, 54],
            [200, 180, 90],
            [33, 66, 99],
            [255, 255, 255],
            [7, 7, 7],
        ] {
            let back = Oklab::from_srgb_u8(rgb).to_srgb_u8();
            for c in 0..3 {
                assert!(
                    (back[c] as i32 - rgb[c] as i32).abs() <= 1,
                    "oklab roundtrip {rgb:?} -> {back:?}"
                );
            }
        }
    }

    #[test]
    fn distance_properties() {
        let a = Oklab::from_srgb_u8([100, 150, 80]);
        let b = Oklab::from_srgb_u8([102, 148, 82]);
        let c = Oklab::from_srgb_u8([200, 40, 40]);
        assert_eq!(a.distance(a), 0.0);
        assert!(a.distance(b) > 0.0 && a.distance(b) < 0.02, "near {}", a.distance(b));
        assert!(a.distance(c) > 0.2, "far {}", a.distance(c));
        assert!((a.distance(b) - b.distance(a)).abs() < 1e-6, "symmetric");
    }

    #[test]
    fn weighted_mean_midpoint() {
        let m = Oklab::weighted_mean(
            &[
                Oklab { l: 0.0, a: 0.0, b: 0.0 },
                Oklab { l: 1.0, a: 0.2, b: -0.2 },
            ],
            &[1.0, 1.0],
        );
        assert!(close(m.l, 0.5, 1e-6) && close(m.a, 0.1, 1e-6) && close(m.b, -0.1, 1e-6));
    }
}
