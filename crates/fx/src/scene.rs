//! The Phase 2b effect test scene (fake data, like Phase 2's fake
//! house): simple colored boxes for the base layer plus placed field
//! tiles at intensities 1–3.
//!
//! This is a STANDALONE TEST HARNESS inside `fx`: the crate may not
//! depend on `crates/engine`, so the base layer is a handful of
//! axis-aligned boxes rasterized by `render.rs`'s box pipeline rather
//! than the engine's mesher output. Wiring these same field/post
//! passes into the real engine renderer is the documented follow-up
//! (docs/phase2b-effects.md).
//!
//! Coordinates: game tiles (x east, y south, z up) map to renderer
//! world (x, z_up, -y_south), the engine's convention.

use glam::Vec3;

use crate::effect::{demo_defs, EffectDef};

/// One axis-aligned colored box in world space.
#[derive(Debug, Clone, Copy)]
pub struct BoxPrim {
    pub min: Vec3,
    pub max: Vec3,
    pub color: [f32; 3],
}

fn bx(min: [f32; 3], max: [f32; 3], color: [f32; 3]) -> BoxPrim {
    BoxPrim {
        min: Vec3::from_array(min),
        max: Vec3::from_array(max),
        color,
    }
}

/// A field volume placed on a game tile.
#[derive(Debug, Clone)]
pub struct PlacedField {
    /// Game tile (x east, y south, z-level).
    pub tile: [i32; 3],
    pub def: EffectDef,
    pub intensity: u8,
}

impl PlacedField {
    /// World-space center of the volume (sits on the floor, y-up).
    pub fn world_center(&self) -> Vec3 {
        Vec3::new(
            self.tile[0] as f32 + 0.5,
            0.55 + self.tile[2] as f32,
            -(self.tile[1] as f32 + 0.5),
        )
    }
}

/// The effect test scene: a small house-like backdrop (floor, bright
/// north wall to judge obscurance against, side walls, crates) and a
/// row of smoke / fire / gas tiles at intensities 1, 2, 3.
pub fn test_scene() -> (Vec<BoxPrim>, Vec<PlacedField>) {
    let boxes = vec![
        // Ground + wood floor.
        bx([0.0, -0.2, -10.0], [14.0, 0.0, 0.0], [0.42, 0.32, 0.20]),
        bx([0.4, 0.0, -9.6], [13.6, 0.06, -0.4], [0.63, 0.43, 0.24]),
        // Bright cream backdrop wall (north, game y = 0 row).
        bx([0.0, 0.0, -1.0], [14.0, 3.0, 0.0], [0.93, 0.89, 0.79]),
        // Side walls.
        bx([0.0, 0.0, -10.0], [0.6, 1.6, -1.0], [0.55, 0.36, 0.28]),
        bx([13.4, 0.0, -10.0], [14.0, 1.6, -1.0], [0.55, 0.36, 0.28]),
        // Crates for depth cues.
        bx([1.0, 0.06, -8.6], [2.2, 1.1, -7.4], [0.50, 0.38, 0.22]),
        bx([11.8, 0.06, -8.6], [13.0, 1.1, -7.4], [0.50, 0.38, 0.22]),
        // A dark doorway block in the backdrop so fire bloom has a
        // dark neighbour to spill onto.
        bx([6.2, 0.0, -1.05], [7.8, 2.2, -0.95], [0.10, 0.09, 0.10]),
    ];
    let defs = demo_defs();
    let get = |id: &str| defs.iter().find(|d| d.field_id == id).unwrap().clone();
    let mut fields = Vec::new();
    // Row at game y = 4: smoke 1-3, fire 1-3, gas 1-3.
    for (i, intensity) in [1u8, 2, 3].iter().enumerate() {
        fields.push(PlacedField {
            tile: [1 + i as i32, 4, 0],
            def: get("fd_smoke"),
            intensity: *intensity,
        });
        fields.push(PlacedField {
            tile: [5 + i as i32, 4, 0],
            def: get("fd_fire"),
            intensity: *intensity,
        });
        fields.push(PlacedField {
            tile: [9 + i as i32, 4, 0],
            def: get("fd_gas"),
            intensity: *intensity,
        });
    }
    (boxes, fields)
}

/// Night fire showcase scene (added 2026-11-15 for the flame redo,
/// against Joey's HD-2D reference screenshots): a dark street where
/// the fires are the light source — dark ground and walls, a row of
/// burning tiles at intensities 1–3, crate silhouettes to catch the
/// glow. Same tile→world mapping as `test_scene`.
pub fn fire_scene() -> (Vec<BoxPrim>, Vec<PlacedField>) {
    let boxes = vec![
        // Dark ground + street slab.
        bx([0.0, -0.2, -10.0], [14.0, 0.0, 0.0], [0.13, 0.11, 0.10]),
        bx([0.4, 0.0, -9.6], [13.6, 0.06, -0.4], [0.20, 0.16, 0.13]),
        // Dark backdrop wall.
        bx([0.0, 0.0, -1.0], [14.0, 3.0, 0.0], [0.16, 0.13, 0.12]),
        // Side walls.
        bx([0.0, 0.0, -10.0], [0.6, 1.6, -1.0], [0.15, 0.12, 0.11]),
        bx([13.4, 0.0, -10.0], [14.0, 1.6, -1.0], [0.15, 0.12, 0.11]),
        // Crate silhouettes for the glow to land on.
        bx([1.0, 0.06, -8.6], [2.2, 1.1, -7.4], [0.19, 0.15, 0.11]),
        bx([11.8, 0.06, -8.6], [13.0, 1.1, -7.4], [0.19, 0.15, 0.11]),
    ];
    let defs = demo_defs();
    let fire = defs.iter().find(|d| d.field_id == "fd_fire").unwrap().clone();
    let mut fields = Vec::new();
    // A burning row: a big central blaze (3,3), neighbours at 2,
    // one small fire at 1 off to the side.
    for (x, intensity) in [(3, 2u8), (4, 3), (5, 3), (6, 2), (9, 3), (11, 1)] {
        fields.push(PlacedField {
            tile: [x, 4, 0],
            def: fire.clone(),
            intensity,
        });
    }
    (boxes, fields)
}

/// Minimal scene for the intensity gates: one field volume centered
/// in front of a pure-white backdrop, nothing else to hide behind.
pub fn single_field_scene(def: EffectDef, intensity: u8) -> (Vec<BoxPrim>, Vec<PlacedField>) {
    let boxes = vec![
        bx([-3.0, -0.2, -4.0], [4.0, 0.0, 2.0], [0.35, 0.35, 0.35]),
        // White backdrop directly behind the volume.
        bx([-3.0, 0.0, -3.4], [4.0, 3.0, -3.0], [1.0, 1.0, 1.0]),
    ];
    let fields = vec![PlacedField {
        tile: [0, 2, 0],
        def,
        intensity,
    }];
    (boxes, fields)
}
