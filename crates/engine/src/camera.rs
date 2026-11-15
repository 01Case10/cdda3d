//! Orbit/follow camera (gate test 2.7).
//!
//! World space is the mesher's Y-up space: grid `(x east, y south, z up)`
//! maps to world `(x, z, -y)`. The camera orbits its follow target (the
//! player position) at a clamped pitch and zoom distance.

use glam::{Mat4, Vec3};

pub const PITCH_MIN_DEG: f32 = 20.0;
pub const PITCH_MAX_DEG: f32 = 85.0;
pub const DIST_MIN: f32 = 2.0;
pub const DIST_MAX: f32 = 160.0;
pub const DEFAULT_PITCH_DEG: f32 = 45.0;
pub const DEFAULT_DISTANCE: f32 = 28.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrbitCamera {
    /// Follow target in world space (player position).
    pub target: Vec3,
    /// Orbit yaw in degrees, normalized to [0, 360).
    pub yaw_deg: f32,
    /// Orbit pitch (elevation) in degrees, clamped to [20, 85].
    pub pitch_deg: f32,
    /// Distance from target, clamped to [2, 160].
    pub distance: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            yaw_deg: 0.0,
            pitch_deg: DEFAULT_PITCH_DEG,
            distance: DEFAULT_DISTANCE,
        }
    }
}

fn sanitize(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        fallback
    }
}

impl OrbitCamera {
    pub fn new(target: Vec3) -> Self {
        Self {
            target,
            ..Self::default()
        }
    }

    /// Apply orbit input (degrees / distance units), then re-clamp.
    /// Non-finite input is ignored rather than poisoning the camera.
    pub fn orbit(&mut self, yaw_delta_deg: f32, pitch_delta_deg: f32, zoom_delta: f32) {
        if yaw_delta_deg.is_finite() {
            self.yaw_deg += yaw_delta_deg;
        }
        if pitch_delta_deg.is_finite() {
            self.pitch_deg += pitch_delta_deg;
        }
        if zoom_delta.is_finite() {
            self.distance += zoom_delta;
        }
        self.clamp();
    }

    /// Move the follow target (player moved / target dragged).
    pub fn follow(&mut self, target: Vec3) {
        if target.is_finite() {
            self.target = target;
        }
    }

    /// Enforce all limits and finiteness. Idempotent; called after any
    /// mutation, and the matrix builders call it defensively via
    /// `sanitized()`.
    pub fn clamp(&mut self) {
        self.yaw_deg = sanitize(self.yaw_deg, 0.0).rem_euclid(360.0);
        self.pitch_deg =
            sanitize(self.pitch_deg, DEFAULT_PITCH_DEG).clamp(PITCH_MIN_DEG, PITCH_MAX_DEG);
        self.distance = sanitize(self.distance, DEFAULT_DISTANCE).clamp(DIST_MIN, DIST_MAX);
        if !self.target.is_finite() {
            self.target = Vec3::ZERO;
        }
    }

    fn sanitized(mut self) -> Self {
        self.clamp();
        self
    }

    /// Eye position in world space.
    pub fn eye(&self) -> Vec3 {
        let c = self.sanitized();
        let yaw = c.yaw_deg.to_radians();
        let pitch = c.pitch_deg.to_radians();
        let offset = Vec3::new(
            pitch.cos() * yaw.sin(),
            pitch.sin(),
            pitch.cos() * yaw.cos(),
        ) * c.distance;
        c.target + offset
    }

    pub fn view_matrix(&self) -> Mat4 {
        let c = self.sanitized();
        Mat4::look_at_rh(c.eye(), c.target, Vec3::Y)
    }

    pub fn projection_matrix(&self, aspect: f32) -> Mat4 {
        let aspect = if aspect.is_finite() && aspect > 0.0 {
            aspect
        } else {
            1.0
        };
        Mat4::perspective_rh(60f32.to_radians(), aspect, 0.1, 600.0)
    }

    pub fn view_projection(&self, aspect: f32) -> Mat4 {
        self.projection_matrix(aspect) * self.view_matrix()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_2_7_pitch_and_zoom_clamps() {
        let mut cam = OrbitCamera::default();
        cam.orbit(0.0, -1000.0, 0.0);
        assert_eq!(cam.pitch_deg, PITCH_MIN_DEG);
        cam.orbit(0.0, 1000.0, 0.0);
        assert_eq!(cam.pitch_deg, PITCH_MAX_DEG);
        cam.orbit(0.0, 0.0, -10000.0);
        assert_eq!(cam.distance, DIST_MIN);
        cam.orbit(0.0, 0.0, 10000.0);
        assert_eq!(cam.distance, DIST_MAX);
    }

    #[test]
    fn test_2_7_yaw_wraps() {
        let mut cam = OrbitCamera::default();
        cam.orbit(370.0, 0.0, 0.0);
        assert!((cam.yaw_deg - 10.0).abs() < 1e-4);
        cam.orbit(-20.0, 0.0, 0.0);
        assert!((cam.yaw_deg - 350.0).abs() < 1e-4);
    }

    #[test]
    fn test_2_7_no_nan_at_extremes() {
        let mut cam = OrbitCamera::default();
        cam.orbit(f32::INFINITY, f32::NAN, f32::NEG_INFINITY);
        cam.follow(Vec3::new(f32::NAN, 0.0, 0.0));
        let vp = cam.view_projection(16.0 / 9.0);
        assert!(vp.to_cols_array().iter().all(|v| v.is_finite()));
        assert!(cam.eye().is_finite());
        // Even a hand-corrupted camera produces finite matrices.
        let mut bad = OrbitCamera::default();
        bad.pitch_deg = f32::NAN;
        bad.distance = f32::INFINITY;
        bad.target = Vec3::splat(f32::NAN);
        let vp = bad.view_projection(0.0);
        assert!(vp.to_cols_array().iter().all(|v| v.is_finite()));
    }

    #[test]
    fn test_2_7_follow_target_and_eye_distance() {
        let mut cam = OrbitCamera::new(Vec3::new(5.0, 1.0, -5.0));
        cam.distance = 10.0;
        let eye = cam.eye();
        assert!((eye.distance(cam.target) - 10.0).abs() < 1e-3);
        cam.follow(Vec3::new(6.0, 1.0, -5.0));
        assert_eq!(cam.target, Vec3::new(6.0, 1.0, -5.0));
        // Pitch clamp means the eye is always above the target plane.
        assert!(cam.eye().y > cam.target.y);
    }
}
