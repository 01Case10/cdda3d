//! Effect-parameter extraction (plan Phase 2b, gate 2b.1).
//!
//! Joey's direction, non-negotiable: field sprites are NEVER rendered.
//! A sprite contributes ONLY extracted parameters — palette, emissive
//! tint, alpha/density profile — and a procedural shader in the `fx`
//! crate produces the actual visuals from those parameters.
//!
//! For each field type this module emits one `effects.json` entry:
//! base color, emissive color + strength, density scale, noise
//! scale/speed, buoyancy/drift, and a render class. Classification is
//! by field id substring with the documented mapping in
//! [`classify_field`]. Every requested field type resolves to an entry
//! or an explicit logged fallback — never a sprite, never silence.
//!
//! Determinism: ids are processed in sorted order, all statistics are
//! order-independent integer accumulations converted to floats at the
//! end, and serialization uses a fixed struct field order — two runs
//! over the same tileset are byte-identical.

use image::{Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tileset::{resolve, Stage, Tileset};

use crate::{composite_entry, VoxelizerConfig};

/// How the fx shader should render a field. Serialized lowercase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RenderClass {
    Smoke,
    Flame,
    Gas,
    Liquid,
    Energy,
    Generic,
}

impl RenderClass {
    pub fn name(&self) -> &'static str {
        match self {
            RenderClass::Smoke => "smoke",
            RenderClass::Flame => "flame",
            RenderClass::Gas => "gas",
            RenderClass::Liquid => "liquid",
            RenderClass::Energy => "energy",
            RenderClass::Generic => "generic",
        }
    }
    pub fn parse(s: &str) -> Option<RenderClass> {
        match s {
            "smoke" => Some(RenderClass::Smoke),
            "flame" => Some(RenderClass::Flame),
            "gas" => Some(RenderClass::Gas),
            "liquid" => Some(RenderClass::Liquid),
            "energy" => Some(RenderClass::Energy),
            "generic" => Some(RenderClass::Generic),
            _ => None,
        }
    }
}

/// Classify a field id into a render class. Documented mapping
/// (first match wins, matched on the lowercase id):
///
/// | substring(s)                                   | class   |
/// |------------------------------------------------|---------|
/// | `fire`, `flame`                                | flame   |
/// | `smoke` (incl. cig/crack/meth/weed smoke)      | smoke   |
/// | `gas`, `fungal`, `haze`, `dust`, `spore`, `seep`, `stench`, `tear` | gas |
/// | `blood`, `acid`, `bile`, `sludge`, `fluid`, `water`, `goo`, `sap`, `ink` | liquid |
/// | `electric`, `light`, `spotlight`, `plasma`, `spark`, `rad` (radiation) | energy |
/// | anything else (`web`, `splinters`, unknown…)   | generic |
pub fn classify_field(field_id: &str) -> RenderClass {
    let id = field_id.to_ascii_lowercase();
    let has = |pats: &[&str]| pats.iter().any(|p| id.contains(p));
    if has(&["fire", "flame"]) {
        RenderClass::Flame
    } else if has(&["smoke"]) {
        RenderClass::Smoke
    } else if has(&["gas", "fungal", "haze", "dust", "spore", "seep", "stench", "tear"]) {
        RenderClass::Gas
    } else if has(&[
        "blood", "acid", "bile", "sludge", "fluid", "water", "goo", "sap", "ink",
    ]) {
        RenderClass::Liquid
    } else if has(&["electric", "light", "plasma", "spark", "radiation"]) {
        RenderClass::Energy
    } else {
        RenderClass::Generic
    }
}

/// Per-class shader defaults: (density_scale, noise_scale, noise_speed,
/// buoyancy [x,y,z] in tiles/sec with +z up in game space mapped to
/// world +y by the renderer, drift_x). Sprite statistics modulate
/// density and emissive; these are the class character.
fn class_defaults(class: RenderClass) -> ClassDefaults {
    match class {
        RenderClass::Smoke => ClassDefaults {
            density_scale: 1.25,
            noise_scale: 2.0,
            noise_speed: 0.25,
            buoyancy: [0.05, 0.0, 0.35],
            base_emissive_strength: 0.0,
        },
        RenderClass::Flame => ClassDefaults {
            density_scale: 0.9,
            noise_scale: 3.5,
            noise_speed: 1.6,
            buoyancy: [0.0, 0.0, 0.9],
            base_emissive_strength: 2.2,
        },
        RenderClass::Gas => ClassDefaults {
            density_scale: 0.55,
            noise_scale: 1.6,
            noise_speed: 0.18,
            buoyancy: [0.02, 0.0, -0.05],
            base_emissive_strength: 0.0,
        },
        RenderClass::Liquid => ClassDefaults {
            density_scale: 0.8,
            noise_scale: 1.2,
            noise_speed: 0.1,
            buoyancy: [0.0, 0.0, -0.4],
            base_emissive_strength: 0.0,
        },
        RenderClass::Energy => ClassDefaults {
            density_scale: 0.7,
            noise_scale: 4.0,
            noise_speed: 2.2,
            buoyancy: [0.0, 0.0, 0.1],
            base_emissive_strength: 1.8,
        },
        RenderClass::Generic => ClassDefaults {
            density_scale: 0.7,
            noise_scale: 2.0,
            noise_speed: 0.4,
            buoyancy: [0.0, 0.0, 0.0],
            base_emissive_strength: 0.0,
        },
    }
}

struct ClassDefaults {
    density_scale: f32,
    noise_scale: f32,
    noise_speed: f32,
    buoyancy: [f32; 3],
    base_emissive_strength: f32,
}

/// One `effects.json` entry. Field order is the serialization order —
/// do not reorder (byte-identical reruns depend on it, as does the
/// `fx` crate's mirror struct).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectParams {
    pub field_id: String,
    pub render_class: RenderClass,
    /// Alpha-weighted mean sprite color, sRGB 0..=1.
    pub base_color: [f32; 3],
    /// Hottest-pixel color (drives bloom tint), sRGB 0..=1.
    pub emissive_color: [f32; 3],
    pub emissive_strength: f32,
    /// Multiplier on the intensity→density curve in the fx crate.
    pub density_scale: f32,
    /// fbm sampling frequency (per tile).
    pub noise_scale: f32,
    /// fbm animation speed multiplier.
    pub noise_speed: f32,
    /// [x, y, z] drift of the noise field, tiles/sec (z up).
    pub buoyancy: [f32; 3],
    /// Fraction of sprite texels with alpha > 0, alpha-weighted 0..=1.
    pub alpha_coverage: f32,
    /// True when no sprite could be sampled and class defaults with a
    /// neutral palette were used (always logged by the caller).
    pub fallback: bool,
}

/// Statistics sampled directly from a sprite image. This is the ONLY
/// thing a sprite contributes to an effect (gate 2b.1/2b.2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpriteStats {
    /// Alpha-weighted mean color, sRGB 0..=1 (premultiplied-clean:
    /// fully transparent texels contribute nothing, so poison colors
    /// stored under alpha=0 cannot leak into the palette).
    pub mean_color: [f32; 3],
    /// Color of the highest-luminance texel with alpha >= 128.
    pub hottest_color: [f32; 3],
    /// Alpha-weighted coverage 0..=1.
    pub coverage: f32,
    /// Mean luminance of visible texels 0..=1.
    pub mean_luminance: f32,
}

/// Sample a sprite. Integer accumulation keeps this deterministic and
/// exactly reproducible by an independent sampler (gate 2b.1 asserts
/// the palette against a direct sample computed in the test).
pub fn sample_sprite(img: &RgbaImage) -> SpriteStats {
    let mut sum = [0u64; 3];
    let mut alpha_sum: u64 = 0;
    let mut lum_sum: u64 = 0;
    let mut best_lum: i64 = -1;
    let mut hottest = [0u8; 3];
    for px in img.pixels() {
        let Rgba([r, g, b, a]) = *px;
        let a = a as u64;
        if a == 0 {
            continue;
        }
        sum[0] += r as u64 * a;
        sum[1] += g as u64 * a;
        sum[2] += b as u64 * a;
        alpha_sum += a;
        let lum = (r as u64 * 299 + g as u64 * 587 + b as u64 * 114) / 1000;
        lum_sum += lum * a;
        if a >= 128 && lum as i64 > best_lum {
            best_lum = lum as i64;
            hottest = [r, g, b];
        }
    }
    let total_texels = (img.width() as u64 * img.height() as u64).max(1);
    if alpha_sum == 0 {
        return SpriteStats {
            mean_color: [0.5, 0.5, 0.5],
            hottest_color: [0.5, 0.5, 0.5],
            coverage: 0.0,
            mean_luminance: 0.0,
        };
    }
    SpriteStats {
        mean_color: [
            sum[0] as f32 / alpha_sum as f32 / 255.0,
            sum[1] as f32 / alpha_sum as f32 / 255.0,
            sum[2] as f32 / alpha_sum as f32 / 255.0,
        ],
        hottest_color: [
            hottest[0] as f32 / 255.0,
            hottest[1] as f32 / 255.0,
            hottest[2] as f32 / 255.0,
        ],
        coverage: alpha_sum as f32 / (total_texels as f32 * 255.0),
        mean_luminance: lum_sum as f32 / alpha_sum as f32 / 255.0,
    }
}

/// Build the effect entry for one field id from sampled sprite stats
/// (or `None` for the fallback path), applying overrides.
pub fn effect_from_stats(
    field_id: &str,
    stats: Option<SpriteStats>,
    cfg: &VoxelizerConfig,
) -> EffectParams {
    let ov = cfg.overrides.entries.get(field_id);
    let class = ov
        .and_then(|o| o.effect_class.as_deref())
        .and_then(RenderClass::parse)
        .unwrap_or_else(|| classify_field(field_id));
    let d = class_defaults(class);
    let (base_color, emissive_color, coverage, lum) = match stats {
        Some(s) => (s.mean_color, s.hottest_color, s.coverage, s.mean_luminance),
        None => ([0.55, 0.55, 0.55], [0.55, 0.55, 0.55], 0.5, 0.3),
    };
    // Density: class scale modulated by how much of the sprite is
    // actually present (a wispy sprite => a wispy volume).
    let mut density = d.density_scale * (0.45 + 0.55 * coverage);
    if let Some(v) = ov.and_then(|o| o.effect_density) {
        density = v;
    }
    // Emissive: class base, boosted by sprite brightness for
    // flame/energy so dim sprites bloom less than bright ones.
    let mut emissive = if d.base_emissive_strength > 0.0 {
        d.base_emissive_strength * (0.5 + lum)
    } else {
        0.0
    };
    if let Some(v) = ov.and_then(|o| o.effect_emissive) {
        emissive = v;
    }
    let mut noise_scale = d.noise_scale;
    if let Some(v) = ov.and_then(|o| o.effect_noise_scale) {
        noise_scale = v;
    }
    EffectParams {
        field_id: field_id.to_string(),
        render_class: class,
        base_color,
        emissive_color,
        emissive_strength: round4(emissive),
        density_scale: round4(density),
        noise_scale: round4(noise_scale),
        noise_speed: d.noise_speed,
        buoyancy: d.buoyancy,
        alpha_coverage: round4(coverage),
        fallback: stats.is_none(),
    }
}

/// Round to 4 decimals so float formatting in JSON is stable across
/// runs/platforms (part of the byte-identical guarantee).
fn round4(v: f32) -> f32 {
    (v * 10000.0).round() / 10000.0
}

/// Extract effects for an explicit list of field ids against a loaded
/// tileset. Returns (entries sorted by field_id, fallback log lines).
/// Ids are deduplicated and sorted first — caller order never leaks
/// into the output.
pub fn extract_effects(
    ts: &Tileset,
    cfg: &VoxelizerConfig,
    field_ids: &[String],
) -> (Vec<EffectParams>, Vec<String>) {
    let mut ids: Vec<String> = field_ids.to_vec();
    ids.sort();
    ids.dedup();
    let mut out = Vec::new();
    let mut log = Vec::new();
    for id in &ids {
        let res = resolve(ts, id, &cfg.looks_like, None, &cfg.season);
        let stats = if res.stage == Stage::Missing || res.stage == Stage::Filler {
            None
        } else {
            res.entry
                .as_ref()
                .and_then(|e| composite_entry(ts, e))
                .map(|img| sample_sprite(&img))
        };
        if stats.is_none() {
            log.push(format!(
                "{id}\tno sprite resolvable — class-default fallback effect (never a sprite)"
            ));
        }
        out.push(effect_from_stats(id, stats, cfg));
    }
    (out, log)
}

/// All field-type ids present in a tileset: entry ids starting with
/// `fd_` (CDDA's field id namespace), sorted.
pub fn field_ids_in_tileset(ts: &Tileset) -> Vec<String> {
    let mut ids: Vec<String> = ts
        .entries
        .keys()
        .filter(|id| id.starts_with("fd_"))
        .cloned()
        .collect();
    ids.sort();
    ids
}

/// Serialize an effects table exactly as it lands in `effects.json`.
/// A BTreeMap wrapper keeps the top level an object keyed by field id
/// (stable key order) with a format marker.
pub fn effects_json(entries: &[EffectParams]) -> String {
    let mut map = BTreeMap::new();
    for e in entries {
        map.insert(e.field_id.clone(), e.clone());
    }
    let doc = serde_json::json!({
        "format": "cdda-voxel-effects-v1",
        "note": "Shader-effect parameters extracted from field sprites. Sprites parameterize effects; they are never rendered as field visuals (Phase 2b).",
        "effects": map,
    });
    serde_json::to_string_pretty(&doc).unwrap() + "\n"
}

/// Run the effect pass and write `effects.json` (+ `effects_fallback.log`)
/// into `out_dir`. Kept separate from [`crate::run`] so the Phase 5
/// golden file set is untouched; the voxelizer bin calls this when
/// `--effects` is passed.
pub fn run_effects(
    ts: &Tileset,
    cfg: &VoxelizerConfig,
    field_ids: &[String],
    out_dir: &std::path::Path,
) -> Result<(Vec<EffectParams>, Vec<String>), String> {
    std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
    let (entries, log) = extract_effects(ts, cfg, field_ids);
    std::fs::write(out_dir.join("effects.json"), effects_json(&entries))
        .map_err(|e| e.to_string())?;
    let mut log_text = log.join("\n");
    if !log_text.is_empty() {
        log_text.push('\n');
    }
    std::fs::write(out_dir.join("effects_fallback.log"), log_text).map_err(|e| e.to_string())?;
    Ok((entries, log))
}
