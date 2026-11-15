//! Effect definitions: the `effects.json` contract.
//!
//! This struct mirrors `voxelizer::effects::EffectParams` field for
//! field (the fx crate deliberately does not depend on the voxelizer —
//! the JSON file is the contract between the offline pipeline and the
//! renderer, exactly as `materials.json` is for blocks). Sprites
//! parameterize these values; no sprite data ever reaches the shader.

use serde::{Deserialize, Serialize};

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
    /// Shader class id (must match the WGSL in `render.rs`).
    pub fn id(&self) -> u32 {
        match self {
            RenderClass::Smoke => 0,
            RenderClass::Flame => 1,
            RenderClass::Gas => 2,
            RenderClass::Liquid => 3,
            RenderClass::Energy => 4,
            RenderClass::Generic => 5,
        }
    }
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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectDef {
    pub field_id: String,
    pub render_class: RenderClass,
    pub base_color: [f32; 3],
    pub emissive_color: [f32; 3],
    pub emissive_strength: f32,
    pub density_scale: f32,
    pub noise_scale: f32,
    pub noise_speed: f32,
    /// Game-space drift [x, y, z], z up. Converted to the renderer's
    /// Y-up world space by [`EffectDef::world_drift`].
    pub buoyancy: [f32; 3],
    pub alpha_coverage: f32,
    pub fallback: bool,
}

impl EffectDef {
    /// Buoyancy/drift in renderer world space (x east, y up, z toward
    /// viewer-south): game (x, y_south, z_up) -> world (x, z_up, -y).
    /// The noise field scrolls opposite the drift (the pattern moves
    /// with the drift), handled by the density functions.
    pub fn world_drift(&self) -> glam::Vec3 {
        glam::Vec3::new(self.buoyancy[0], self.buoyancy[2], -self.buoyancy[1])
    }

    /// Deterministic per-type seed so neighbouring tiles of the same
    /// field do not share an identical noise phase.
    pub fn seed(&self) -> f32 {
        let mut h: u32 = 2166136261;
        for b in self.field_id.bytes() {
            h ^= b as u32;
            h = h.wrapping_mul(16777619);
        }
        (h % 1000) as f32 * 0.37
    }
}

/// Load an `effects.json` written by the voxelizer effect pass.
/// Returns defs sorted by field_id.
pub fn load_effects_json(text: &str) -> Result<Vec<EffectDef>, String> {
    #[derive(Deserialize)]
    struct Doc {
        effects: std::collections::BTreeMap<String, EffectDef>,
    }
    let doc: Doc = serde_json::from_str(text).map_err(|e| e.to_string())?;
    Ok(doc.effects.into_values().collect())
}

/// Hand-authored defs for the effect test scene (fake data, like
/// Phase 2's fake house): one per headline class.
pub fn demo_defs() -> Vec<EffectDef> {
    vec![
        EffectDef {
            field_id: "fd_smoke".into(),
            render_class: RenderClass::Smoke,
            base_color: [0.62, 0.62, 0.64],
            emissive_color: [0.62, 0.62, 0.64],
            emissive_strength: 0.0,
            density_scale: 1.25,
            noise_scale: 2.0,
            noise_speed: 0.25,
            buoyancy: [0.05, 0.0, 0.35],
            alpha_coverage: 0.8,
            fallback: false,
        },
        EffectDef {
            field_id: "fd_fire".into(),
            render_class: RenderClass::Flame,
            base_color: [0.95, 0.45, 0.12],
            emissive_color: [1.0, 0.62, 0.18],
            emissive_strength: 2.4,
            density_scale: 0.9,
            noise_scale: 3.5,
            noise_speed: 1.6,
            buoyancy: [0.0, 0.0, 0.9],
            alpha_coverage: 0.7,
            fallback: false,
        },
        EffectDef {
            field_id: "fd_gas".into(),
            render_class: RenderClass::Gas,
            base_color: [0.45, 0.66, 0.38],
            emissive_color: [0.45, 0.66, 0.38],
            emissive_strength: 0.0,
            density_scale: 0.55,
            noise_scale: 1.6,
            noise_speed: 0.18,
            buoyancy: [0.02, 0.0, -0.05],
            alpha_coverage: 0.5,
            fallback: false,
        },
    ]
}
