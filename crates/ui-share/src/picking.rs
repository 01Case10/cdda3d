//! Mouse picking (gate 4.11): a ray from the camera through a
//! pixel, traversed through the voxel grid to a tile coordinate.
//!
//! World convention matches the engine (`crates/engine/src/camera.rs`):
//! grid `(x east, y south, z up)` maps to world `(x, z, -y)` — one
//! tile is one world unit, one z-level is 3 blocks tall in the render
//! world but picking works in *tile* space, so a tile occupies
//! `1 x 3 x 1` world units (x, up, -y). The traversal is
//! Amanatides–Woo DDA over tile cells in that scaled space.

use glam::{Mat4, Vec3};
use proto::{PickResultT, Vec3iT};

use crate::frame::TerrainRect;
use crate::input::InputRouter;

#[derive(Debug, Clone, Copy)]
pub struct PickCamera {
    pub target: Vec3, // world space
    pub yaw_deg: f32,
    pub pitch_deg: f32,
    pub distance: f32,
    pub fov_y_deg: f32,
}
impl Default for PickCamera {
    fn default() -> Self {
        Self { target: Vec3::ZERO, yaw_deg: 0.0, pitch_deg: 45.0, distance: 28.0, fov_y_deg: 60.0 }
    }
}
impl PickCamera {
    pub fn eye(&self) -> Vec3 {
        let yaw = self.yaw_deg.to_radians();
        let pitch = self.pitch_deg.clamp(1.0, 89.0).to_radians();
        self.target + Vec3::new(pitch.cos() * yaw.sin(), pitch.sin(), pitch.cos() * yaw.cos()) * self.distance
    }
    pub fn view_projection(&self, aspect: f32) -> Mat4 {
        Mat4::perspective_rh(self.fov_y_deg.to_radians(), aspect.max(0.01), 0.1, 600.0)
            * Mat4::look_at_rh(self.eye(), self.target, Vec3::Y)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ray { pub origin: Vec3, pub dir: Vec3 }

/// Ray through the center of viewport pixel `(px, py)`. The pixel is
/// in *terrain-rect* space when picking from a UI click: the caller
/// passes the terrain rect as the viewport (origin offset handled by
/// `pick_from_ui_click`); here `(px,py)` is local to a
/// `vp_w x vp_h` viewport.
pub fn ray_from_pixel(cam: &PickCamera, px: f32, py: f32, vp_w: f32, vp_h: f32) -> Ray {
    let aspect = vp_w / vp_h.max(1.0);
    let inv = cam.view_projection(aspect).inverse();
    let ndc = glam::Vec4::new(
        (px + 0.5) / vp_w * 2.0 - 1.0,
        1.0 - (py + 0.5) / vp_h * 2.0,
        -1.0, 1.0,
    );
    let near4 = inv * ndc;
    let near = near4.truncate() / near4.w.max(1e-6);
    let origin = cam.eye();
    Ray { origin, dir: (near - origin).normalize() }
}

/// World point -> tile coordinate (floor in tile space; z is a
/// z-level, 3 world units tall).
pub fn world_to_tile(p: Vec3) -> (i32, i32, i32) {
    (p.x.floor() as i32, (-p.z).floor() as i32, (p.y / 3.0).floor() as i32)
}
pub fn tile_center_world(tile: (i32, i32, i32)) -> Vec3 {
    Vec3::new(tile.0 as f32 + 0.5, tile.2 as f32 * 3.0 + 1.5, -(tile.1 as f32 + 0.5))
}

/// Amanatides–Woo traversal in tile space. `occupied(tile)` reports
/// whether a tile stops the ray (solid terrain/furniture). Returns
/// the first occupied tile and the world-space hit distance.
pub fn traverse_to_tile<F: Fn(i32, i32, i32) -> bool>(ray: &Ray, max_dist: f32, occupied: F) -> Option<((i32, i32, i32), f32)> {
    // Convert the ray into tile space: (x, -z_world, y_world/3).
    let o = Vec3::new(ray.origin.x, -ray.origin.z, ray.origin.y / 3.0);
    let d = Vec3::new(ray.dir.x, -ray.dir.z, ray.dir.y / 3.0);
    if d.length_squared() < 1e-12 { return None; }
    let mut cell = glam::IVec3::new(o.x.floor() as i32, o.y.floor() as i32, o.z.floor() as i32);
    let step = glam::IVec3::new(
        if d.x > 0.0 { 1 } else if d.x < 0.0 { -1 } else { 0 },
        if d.y > 0.0 { 1 } else if d.y < 0.0 { -1 } else { 0 },
        if d.z > 0.0 { 1 } else if d.z < 0.0 { -1 } else { 0 },
    );
    let axis = |oc: f32, dc: f32, c: i32| -> (f32, f32) {
        if dc.abs() < 1e-9 { return (f32::INFINITY, f32::INFINITY); }
        let boundary = if dc > 0.0 { c as f32 + 1.0 } else { c as f32 };
        ((boundary - oc) / dc, 1.0 / dc.abs())
    };
    let (mut tx, dtx) = axis(o.x, d.x, cell.x);
    let (mut ty, dty) = axis(o.y, d.y, cell.y);
    let (mut tz, dtz) = axis(o.z, d.z, cell.z);
    // Tile-space t is not world distance; scale by |d_world|/|d_tile| approx via dir length ratio.
    let world_per_tile_t = ray.dir.length() / d.length();
    let mut t = 0.0f32;
    for _ in 0..1024 {
        if occupied(cell.x, cell.y, cell.z) {
            return Some(((cell.x, cell.y, cell.z), t * world_per_tile_t));
        }
        if tx < ty && tx < tz {
            cell.x += step.x; t = tx; tx += dtx;
        } else if ty < tz {
            cell.y += step.y; t = ty; ty += dty;
        } else {
            cell.z += step.z; t = tz; tz += dtz;
        }
        if t * world_per_tile_t > max_dist { return None; }
    }
    None
}

/// Full click -> pick path (gate 4.11 including the separation
/// rule). `ui_x/ui_y` are display-buffer pixels. Returns `None` when
/// the click is not a world pick at all (UiInput mode, panel click,
/// outside the terrain rect); otherwise a `PickResultT` whose `hit`
/// says whether the ray struck an occupied tile.
pub fn pick_from_ui_click<F: Fn(i32, i32, i32) -> bool>(
    request_id: u64,
    router: &InputRouter,
    cam: &PickCamera,
    ui_x: i32,
    ui_y: i32,
    occupied: F,
) -> Option<PickResultT> {
    if !router.is_world_pick(ui_x, ui_y) { return None; }
    let rect: TerrainRect = router.terrain_rect?;
    let local_x = (ui_x - rect.x) as f32;
    let local_y = (ui_y - rect.y) as f32;
    let ray = ray_from_pixel(cam, local_x, local_y, rect.w as f32, rect.h as f32);
    match traverse_to_tile(&ray, 300.0, occupied) {
        Some((tile, _)) => {
            let mut r = PickResultT::default();
            r.request_id = request_id; r.hit = true;
            let mut p = Vec3iT::default(); p.x = tile.0; p.y = tile.1; p.z = tile.2;
            r.pos = Some(p);
            Some(r)
        }
        None => {
            let mut r = PickResultT::default();
            r.request_id = request_id; r.hit = false;
            r.pos = Some(Vec3iT::default());
            Some(r)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proto::UiMode;

    #[test]
    fn center_ray_hits_target_tile() {
        // Camera looking at the center of tile (5,5,0): a ray through
        // the viewport center must pick exactly that tile.
        let target_tile = (5, 5, 0);
        let cam = PickCamera { target: tile_center_world(target_tile), ..Default::default() };
        let ray = ray_from_pixel(&cam, 199.0, 149.0, 400.0, 300.0);
        let hit = traverse_to_tile(&ray, 300.0, |x, y, z| (x, y, z) == target_tile);
        assert_eq!(hit.map(|h| h.0), Some(target_tile));
    }

    #[test]
    fn panel_click_is_not_a_pick() {
        let tr = TerrainRect { x: 0, y: 0, w: 200, h: 200 };
        let router = InputRouter::new(UiMode::MapInput, Some(tr));
        let cam = PickCamera::default();
        // (250, 50) is in the sidebar panel area, outside the terrain rect.
        assert!(pick_from_ui_click(1, &router, &cam, 250, 50, |_, _, _| true).is_none());
        // UiInput mode: even a terrain-rect click is not a pick.
        let router_ui = InputRouter::new(UiMode::UiInput, Some(tr));
        assert!(pick_from_ui_click(1, &router_ui, &cam, 50, 50, |_, _, _| true).is_none());
    }

    #[test]
    fn world_tile_roundtrip() {
        for t in [(0, 0, 0), (3, 7, 1), (-2, 4, -1)] {
            assert_eq!(world_to_tile(tile_center_world(t)), t);
        }
    }
}
