//! Effect id + kind → transient emitter parameters.
//!
//! Render class ids mirror `fx::effect::RenderClass::id()` (fx
//! crate, Phase 2b): Smoke 0, Flame 1, Gas 2, Liquid 3, Energy 4,
//! Generic 5. A transient emitter is drawn by the same shader
//! classes as the field volumes; these parameters are the whole
//! contract between this crate and that pass.

use proto::EffectKind;

/// Shader parameters for one transient emitter, in the Phase 2b
/// parameter space (colors are linear-ish display values in
/// [0, 1], emissive strength is an HDR multiplier — the post
/// stack's bloom picks up anything above ~1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmitterParams {
    /// fx render class id (see module docs).
    pub render_class: u32,
    pub color: [f32; 3],
    pub emissive: [f32; 3],
    pub emissive_strength: f32,
    /// Nominal emitter extent in tiles (a burst is wider than a
    /// tracer point).
    pub size: f32,
}

pub const CLASS_SMOKE: u32 = 0;
pub const CLASS_FLAME: u32 = 1;
pub const CLASS_GAS: u32 = 2;
pub const CLASS_LIQUID: u32 = 3;
pub const CLASS_ENERGY: u32 = 4;
pub const CLASS_GENERIC: u32 = 5;

/// Parameters for one event. Resolution order: the effect id names
/// a known effect family first (an explosion is a flame burst
/// whatever the wire kind says), then the kind's default. Unknown
/// ids fall back to a visible generic flash — an unrecognized
/// effect must still appear (a silent drop would read as the game
/// not firing), in the same spirit as the voxelizer's magenta
/// placeholder for unknown tile ids.
pub fn params_for(kind: EffectKind, effect_id: &str) -> Option<EmitterParams> {
    // Sound markers have no visual emitter: they are forwarded to
    // the engine's audio hook as data (the Emitter records them
    // with `params: None`) and expire immediately.
    if kind == EffectKind::Sound {
        return None;
    }
    let id = effect_id.to_ascii_lowercase();
    if id.contains("explosion") || id.contains("grenade") || id.contains("bomb") {
        return Some(EmitterParams {
            render_class: CLASS_FLAME,
            color: [0.98, 0.55, 0.16],
            emissive: [1.0, 0.62, 0.18],
            emissive_strength: 3.2,
            size: 1.6,
        });
    }
    if id.contains("muzzle") || id.contains("flash") {
        return Some(EmitterParams {
            render_class: CLASS_ENERGY,
            color: [1.0, 0.85, 0.45],
            emissive: [1.0, 0.8, 0.4],
            emissive_strength: 2.6,
            size: 0.5,
        });
    }
    if id.contains("bullet") || id.contains("shot") || id.contains("tracer") {
        return Some(EmitterParams {
            render_class: CLASS_ENERGY,
            color: [1.0, 0.92, 0.65],
            emissive: [1.0, 0.9, 0.55],
            emissive_strength: 2.0,
            size: 0.22,
        });
    }
    if id.contains("smoke") {
        return Some(EmitterParams {
            render_class: CLASS_SMOKE,
            color: [0.62, 0.62, 0.64],
            emissive: [0.62, 0.62, 0.64],
            emissive_strength: 0.0,
            size: 1.0,
        });
    }
    if id.contains("blood") || id.contains("splash") {
        return Some(EmitterParams {
            render_class: CLASS_LIQUID,
            color: [0.55, 0.06, 0.05],
            emissive: [0.55, 0.06, 0.05],
            emissive_strength: 0.0,
            size: 0.7,
        });
    }
    match kind {
        EffectKind::Explosion => Some(EmitterParams {
            render_class: CLASS_FLAME,
            color: [0.98, 0.55, 0.16],
            emissive: [1.0, 0.62, 0.18],
            emissive_strength: 3.2,
            size: 1.6,
        }),
        EffectKind::Projectile => Some(EmitterParams {
            render_class: CLASS_GENERIC,
            color: [0.85, 0.8, 0.7],
            emissive: [0.85, 0.8, 0.7],
            emissive_strength: 0.4,
            size: 0.3,
        }),
        EffectKind::HitFlash => Some(EmitterParams {
            render_class: CLASS_ENERGY,
            color: [1.0, 1.0, 1.0],
            emissive: [1.0, 1.0, 1.0],
            emissive_strength: 1.8,
            size: 0.6,
        }),
        EffectKind::Animation | EffectKind::Other => Some(EmitterParams {
            render_class: CLASS_GENERIC,
            color: [0.9, 0.9, 0.9],
            emissive: [0.9, 0.9, 0.9],
            emissive_strength: 0.8,
            size: 0.8,
        }),
        // Sound returned None above. Any future/unknown kind value
        // (the wire enum is an open u8) gets the generic flash.
        _ => Some(EmitterParams {
            render_class: CLASS_GENERIC,
            color: [0.9, 0.9, 0.9],
            emissive: [0.9, 0.9, 0.9],
            emissive_strength: 0.8,
            size: 0.8,
        }),
    }
}
