//! Gate 7.1 — light mapping: game `lit_level` + seen float -> presentation
//! brightness, plus visibility states (gate 7.2).
//!
//! Terms, defined on first use:
//! - *lit_level* (`LitLevel` in `schema/cdda_ipc.fbs`): the game's own
//!   per-tile visibility-cache enum — `DARK, LOW, BRIGHT_ONLY, LIT, BRIGHT`
//!   for currently-visible tiles, `MEMORIZED` for remembered-but-not-
//!   currently-visible tiles, `BLANK` for never-seen tiles. It is an enum,
//!   not a 0–255 light value; the game computed it from its FOV/lightmap
//!   and this crate never recomputes FOV.
//! - *seen*: the game's per-tile quantized visibility float, `0..=255`
//!   (`0` = never seen, `255` = fully seen). Within one `lit_level` it
//!   orders tiles by how strongly they are seen.
//! - *tint*: an RGB multiplier carried alongside intensity. The game
//!   computes colored light (`light_color_rgb`, float RGB energy);
//!   v1 quantizes it to one tint per tile, but [`LightSample`] already
//!   carries a full `[f32; 3]` tint so per-tile color needs no breaking
//!   API change later (plan §5 Phase 7, §14 Q7).

use proto::LitLevel;

/// Brightness band `[lo, hi]` for one visible `lit_level`. Bands are
/// pairwise disjoint and ordered, which is what makes the mapping
/// *strictly* monotonic in the enum order for *every* seen value:
/// the brightest `DARK` tile (seen 255) is still darker than the
/// dimmest `LOW` tile (seen 0), and so on up the chain.
fn band(lit: LitLevel) -> Option<(f32, f32)> {
    match lit {
        LitLevel::DARK => Some((0.06, 0.14)),
        LitLevel::LOW => Some((0.18, 0.28)),
        LitLevel::BRIGHT_ONLY => Some((0.34, 0.46)),
        LitLevel::LIT => Some((0.54, 0.70)),
        LitLevel::BRIGHT => Some((0.80, 1.00)),
        // MEMORIZED / BLANK have no visible band; any future/unknown
        // value also fails closed (no band) rather than guessing.
        _ => None,
    }
}

/// Raw intensity for a *visible* level before the MEMORIZED/BLANK
/// presentation is applied. MEMORIZED returns a fixed dim value (its
/// real presentation goes through [`apply_visibility`]); BLANK is 0.
pub fn base_intensity(lit: LitLevel, seen: u8) -> f32 {
    let t = seen as f32 / 255.0;
    match band(lit) {
        Some((lo, hi)) => lo + (hi - lo) * t,
        None => {
            if lit == LitLevel::MEMORIZED {
                0.30
            } else {
                0.0 // BLANK and unknown values fail closed (black)
            }
        }
    }
}

/// One tile's light as the renderers consume it: a scalar intensity
/// plus an RGB tint. The effective per-channel multiplier is
/// `intensity * tint[channel]` — see [`LightSample::multiplier`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightSample {
    pub intensity: f32,
    pub tint: [f32; 3],
}

impl LightSample {
    pub const BLACK: LightSample = LightSample {
        intensity: 0.0,
        tint: [1.0, 1.0, 1.0],
    };
    pub const WHITE_FULL: LightSample = LightSample {
        intensity: 1.0,
        tint: [1.0, 1.0, 1.0],
    };

    pub fn new(intensity: f32, tint: [f32; 3]) -> Self {
        Self {
            intensity: intensity.clamp(0.0, 1.0),
            tint,
        }
    }

    /// Per-channel multiplier to apply to an albedo color.
    pub fn multiplier(self) -> [f32; 3] {
        [
            self.intensity * self.tint[0],
            self.intensity * self.tint[1],
            self.intensity * self.tint[2],
        ]
    }

    /// Multiply an RGB(A) albedo by this sample.
    pub fn apply(self, color: [f32; 4]) -> [f32; 4] {
        let m = self.multiplier();
        [
            (color[0] * m[0]).clamp(0.0, 1.0),
            (color[1] * m[1]).clamp(0.0, 1.0),
            (color[2] * m[2]).clamp(0.0, 1.0),
            color[3],
        ]
    }
    pub fn apply_rgb(self, color: [f32; 3]) -> [f32; 3] {
        let m = self.multiplier();
        [
            (color[0] * m[0]).clamp(0.0, 1.0),
            (color[1] * m[1]).clamp(0.0, 1.0),
            (color[2] * m[2]).clamp(0.0, 1.0),
        ]
    }

    /// Component-wise average of a non-empty slice (smoothing kernel).
    pub fn average(samples: &[LightSample]) -> LightSample {
        if samples.is_empty() {
            return LightSample::BLACK;
        }
        let n = samples.len() as f32;
        let mut s = LightSample {
            intensity: 0.0,
            tint: [0.0; 3],
        };
        for x in samples {
            s.intensity += x.intensity;
            for c in 0..3 {
                s.tint[c] += x.tint[c];
            }
        }
        s.intensity /= n;
        for c in 0..3 {
            s.tint[c] /= n;
        }
        s
    }
}

/// Per-tile light/visibility exactly as exported by the game, plus the
/// engine-side facts smoothing needs: the tile's tint (v1: white
/// unless the caller supplies colored light) and whether the tile
/// *occludes* (holds a full-height opaque block — a wall). Occlusion is
/// geometry, not light: it is how smoothing knows a wall separates two
/// tiles without ever propagating light through it.
///
/// `outside` is the Phase 10 sky-exposure fact (§3.4, decision 4A):
/// the game's own test, `map::is_outside(p) && !map::is_roofed(p)`,
/// carried on the wire as `SubmapBlock.outside` / `TileChange.outside`
/// since slice 10.1 and ingested by the proto adapter since 10.6. It
/// is not light either — it gates how much sky GI a tile may take.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TileLight {
    pub lit: LitLevel,
    pub seen: u8,
    pub tint: [f32; 3],
    pub occluding: bool,
    pub outside: bool,
}

impl TileLight {
    pub const BLANK: TileLight = TileLight {
        lit: LitLevel::BLANK,
        seen: 0,
        tint: [1.0, 1.0, 1.0],
        occluding: false,
        outside: false,
    };
    pub fn new(lit: LitLevel, seen: u8) -> Self {
        Self {
            lit,
            seen,
            tint: [1.0, 1.0, 1.0],
            occluding: false,
            outside: false,
        }
    }
    pub fn occluding(mut self, v: bool) -> Self {
        self.occluding = v;
        self
    }
    pub fn outside(mut self, v: bool) -> Self {
        self.outside = v;
        self
    }
    pub fn with_tint(mut self, tint: [f32; 3]) -> Self {
        self.tint = tint;
        self
    }
    /// Raw sample before visibility presentation / environment.
    pub fn raw_sample(self) -> LightSample {
        LightSample::new(base_intensity(self.lit, self.seen), self.tint)
    }
    pub fn visibility(self) -> Visibility {
        visibility_of(self.lit, self.seen)
    }
}

// ---------------------------------------------------------------------------
// Gate 7.2 — visibility states
// ---------------------------------------------------------------------------

/// The three presentation states both renderers must agree on.
/// This enum *is* the visibility-mask contract: the mesher path drops
/// `Hidden` blocks before meshing and the ray-tracer path excludes
/// `Hidden` voxels from the tree entirely (gate 2c.8's discipline,
/// owned here as pure data).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Visibility {
    /// Currently seen by the game: full presentation.
    Visible,
    /// Remembered but not currently seen: desaturated + darker.
    Memorized,
    /// Never seen (`BLANK`): black / not meshed / not in the RT tree.
    Hidden,
}

/// MEMORIZED presentation factors (gate 7.2, chosen here and pinned by
/// tests): brightness ×0.45, saturation kept at 0.30 (i.e. the color is
/// mixed 70% toward its own luminance before the brightness scale).
pub const MEMORIZED_BRIGHTNESS: f32 = 0.45;
pub const MEMORIZED_SATURATION_KEEP: f32 = 0.30;

pub fn visibility_of(lit: LitLevel, seen: u8) -> Visibility {
    if lit == LitLevel::BLANK {
        return Visibility::Hidden;
    }
    if lit == LitLevel::MEMORIZED {
        return Visibility::Memorized;
    }
    // Only the five known visible levels may be Visible; an unknown
    // future value fails closed as Hidden (never reveal what the
    // game has not clearly marked seen).
    let known_visible = matches!(
        lit,
        LitLevel::DARK | LitLevel::LOW | LitLevel::BRIGHT_ONLY | LitLevel::LIT | LitLevel::BRIGHT
    );
    if !known_visible || seen == 0 {
        Visibility::Hidden
    } else {
        Visibility::Visible
    }
}

/// Whether a tile in this state produces geometry at all.
pub fn is_meshed(v: Visibility) -> bool {
    v != Visibility::Hidden
}
/// Whether a voxel in this state enters the ray-tracer tree.
/// Identical rule to [`is_meshed`] by design — one contract, two
/// renderers.
pub fn include_in_tree(v: Visibility) -> bool {
    v != Visibility::Hidden
}

/// Rec. 709 luminance.
pub fn luminance(c: [f32; 3]) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

/// Apply a visibility state's presentation to an RGB color *before*
/// light intensity is multiplied in (callers use
/// [`presented_sample`] for the combined factor; this helper exists
/// for the desaturation step, which is not expressible as a per-channel
/// multiplier because it mixes channels).
pub fn apply_visibility_rgb(color: [f32; 3], v: Visibility) -> [f32; 3] {
    match v {
        Visibility::Visible => color,
        Visibility::Hidden => [0.0, 0.0, 0.0],
        Visibility::Memorized => {
            let l = luminance(color);
            let k = MEMORIZED_SATURATION_KEEP;
            [
                (l + (color[0] - l) * k) * MEMORIZED_BRIGHTNESS,
                (l + (color[1] - l) * k) * MEMORIZED_BRIGHTNESS,
                (l + (color[2] - l) * k) * MEMORIZED_BRIGHTNESS,
            ]
        }
    }
}

/// The full per-tile presentation sample: raw light, with the
/// MEMORIZED dimming folded into intensity and Hidden forced black.
/// Desaturation itself is *not* folded in here (it mixes channels);
/// bakers apply it via [`apply_visibility_rgb`] on the albedo first,
/// then multiply by this sample. Keeping the two steps separate is
/// what lets a single `LightSample` stay a pure multiplier.
pub fn presented_sample(t: TileLight) -> LightSample {
    match t.visibility() {
        Visibility::Hidden => LightSample::BLACK,
        Visibility::Memorized => {
            // raw MEMORIZED intensity (0.30) *is* the dim level; do not
            // multiply by MEMORIZED_BRIGHTNESS again here — that factor
            // is applied to the albedo in apply_visibility_rgb. To keep
            // the combined result from double-dimming, the sample for a
            // memorized tile is full-intensity white-tinted and the
            // albedo step carries both factors. Tests pin the combined
            // result, not either step alone.
            LightSample::new(1.0, t.tint)
        }
        Visibility::Visible => t.raw_sample(),
    }
}

/// Combined presentation of one albedo under one tile's light, with no
/// environment (day/night/weather) applied. This is the reference
/// composition every baker uses:
/// `presented_color = presented_sample * apply_visibility(albedo)`.
pub fn present_color(albedo: [f32; 3], t: TileLight) -> [f32; 3] {
    let base = apply_visibility_rgb(albedo, t.visibility());
    presented_sample(t).apply_rgb(base)
}
