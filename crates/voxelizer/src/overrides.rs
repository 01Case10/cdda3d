//! `overrides.toml` support (plan §8): hand-tuned per-id entries.
//! Format (one table per id):
//! ```toml
//! [f_bookcase]
//! height = 3
//! mode = "box"
//! ```
//! A minimal TOML subset parser is used (tables, string/integer/array
//! values) so invalid files produce clear, located errors (test 5.9).

use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct OverrideEntry {
    pub height: Option<u32>,
    pub mode: String,
    pub material: Option<String>,
    pub transparency: Option<String>,
    pub tint: [u8; 3],
    // Phase 2b effect tuning (all optional; absent = extracted value).
    pub effect_density: Option<f32>,
    pub effect_emissive: Option<f32>,
    pub effect_noise_scale: Option<f32>,
    pub effect_class: Option<String>,
}

impl OverrideEntry {
    pub fn validate(&self) -> Result<(), String> {
        if !matches!(self.mode.as_str(), "extruded" | "box" | "full_block") {
            return Err(format!(
                "invalid mode {:?} (expected extruded|box|full_block)",
                self.mode
            ));
        }
        if let Some(h) = self.height {
            if h == 0 || h > 9 {
                return Err(format!("invalid height {h} (expected 1..=9)"));
            }
        }
        if let Some(t) = &self.transparency {
            if !matches!(t.as_str(), "opaque" | "cutout" | "transparent") {
                return Err(format!("invalid transparency {t:?}"));
            }
        }
        if let Some(c) = &self.effect_class {
            if !matches!(
                c.as_str(),
                "smoke" | "flame" | "gas" | "liquid" | "energy" | "generic"
            ) {
                return Err(format!("invalid effect_class {c:?}"));
            }
        }
        for (name, v) in [
            ("effect_density", self.effect_density),
            ("effect_emissive", self.effect_emissive),
            ("effect_noise_scale", self.effect_noise_scale),
        ] {
            if let Some(v) = v {
                if !(0.0..=10.0).contains(&v) || !v.is_finite() {
                    return Err(format!("invalid {name} {v} (expected 0..=10)"));
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Overrides {
    pub entries: BTreeMap<String, OverrideEntry>,
}

pub fn parse_overrides(text: &str) -> Result<Overrides, String> {
    let mut out = Overrides::default();
    let mut current: Option<String> = None;
    for (lineno, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            if !line.ends_with(']') {
                return Err(format!("line {}: malformed table header", lineno + 1));
            }
            let name = line[1..line.len() - 1].trim().to_string();
            if name.is_empty() {
                return Err(format!("line {}: empty table name", lineno + 1));
            }
            current = Some(name.clone());
            out.entries.entry(name).or_insert(OverrideEntry {
                mode: "extruded".into(),
                tint: [255, 255, 255],
                ..Default::default()
            });
            continue;
        }
        let cur = current
            .clone()
            .ok_or_else(|| format!("line {}: key outside any table", lineno + 1))?;
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| format!("line {}: expected key = value", lineno + 1))?;
        let key = key.trim();
        let value = value.trim();
        let entry = out.entries.get_mut(&cur).unwrap();
        match key {
            "height" => {
                entry.height = Some(
                    value
                        .parse::<u32>()
                        .map_err(|_| format!("line {}: height is not an integer", lineno + 1))?,
                );
            }
            "mode" => {
                entry.mode = unquote(value)
                    .ok_or_else(|| format!("line {}: mode must be a quoted string", lineno + 1))?;
            }
            "material" => {
                entry.material = Some(
                    unquote(value)
                        .ok_or_else(|| format!("line {}: material must be quoted", lineno + 1))?,
                );
            }
            "transparency" => {
                entry.transparency =
                    Some(unquote(value).ok_or_else(|| {
                        format!("line {}: transparency must be quoted", lineno + 1)
                    })?);
            }
            "effect_density" | "effect_emissive" | "effect_noise_scale" => {
                let v = value
                    .parse::<f32>()
                    .map_err(|_| format!("line {}: {key} is not a number", lineno + 1))?;
                match key {
                    "effect_density" => entry.effect_density = Some(v),
                    "effect_emissive" => entry.effect_emissive = Some(v),
                    _ => entry.effect_noise_scale = Some(v),
                }
            }
            "effect_class" => {
                entry.effect_class = Some(
                    unquote(value).ok_or_else(|| {
                        format!("line {}: effect_class must be quoted", lineno + 1)
                    })?,
                );
            }
            "tint" => {
                let inner = value
                    .strip_prefix('[')
                    .and_then(|v| v.strip_suffix(']'))
                    .ok_or_else(|| format!("line {}: tint must be [r, g, b]", lineno + 1))?;
                let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
                if parts.len() != 3 {
                    return Err(format!("line {}: tint must have 3 components", lineno + 1));
                }
                let mut tint = [0u8; 3];
                for (i, p) in parts.iter().enumerate() {
                    tint[i] = p
                        .parse::<u8>()
                        .map_err(|_| format!("line {}: tint component not 0..=255", lineno + 1))?;
                }
                entry.tint = tint;
            }
            other => {
                return Err(format!(
                    "line {}: unknown override key {other:?}",
                    lineno + 1
                ));
            }
        }
    }
    for (id, e) in &out.entries {
        e.validate().map_err(|m| format!("override [{id}]: {m}"))?;
    }
    Ok(out)
}

fn unquote(v: &str) -> Option<String> {
    let v = v.trim();
    if v.len() >= 2 && v.starts_with('"') && v.ends_with('"') {
        Some(v[1..v.len() - 1].to_string())
    } else {
        None
    }
}
