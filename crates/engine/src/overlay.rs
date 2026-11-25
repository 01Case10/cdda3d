//! Debug-overlay state, frame pacing, FPS counter, and the resize
//! decision used by the windowed app (gate tests 2.11 + frame pacing).
//!
//! This is deliberately *state and decisions*, unit-tested in the lib;
//! the windowed bin renders the summary into the window title (no GUI
//! toolkit in the MVP) and the capture path prints it to stdout.

use std::time::{Duration, Instant};

use mesher::mesh::PassMeshes;

/// Aggregate mesh statistics for the debug overlay.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MeshStats {
    pub chunks: usize,
    pub vertices: usize,
    pub quads: usize,
    pub triangles: usize,
}

impl MeshStats {
    pub fn from_pass_meshes(chunks: usize, meshes: &PassMeshes) -> Self {
        Self {
            chunks,
            vertices: meshes.total_vertices(),
            quads: meshes.total_quads(),
            triangles: meshes.meshes.iter().map(|m| m.triangle_count()).sum(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DebugOverlay {
    pub show_chunk_borders: bool,
    pub show_stats: bool,
    pub stats: MeshStats,
    pub fps: f32,
    /// RT frames report no mesh stats (the raster counters
    /// stay zero — the title of the 2026-11-25 Launch-Sim
    /// screenshot claimed "chunks 0 | verts 0 | quads 0"
    /// over a fully rendered world). When set, the summary
    /// reports the traced voxel (leaf) count instead.
    pub rt_voxels: Option<usize>,
}

impl DebugOverlay {
    pub fn new() -> Self {
        Self {
            show_chunk_borders: false,
            show_stats: true,
            stats: MeshStats::default(),
            fps: 0.0,
            rt_voxels: None,
        }
    }

    pub fn summary(&self) -> String {
        if let Some(voxels) = self.rt_voxels {
            return format!(
                "fps {:.0} | chunks {} | rt voxels {} | borders {}",
                self.fps,
                self.stats.chunks,
                voxels,
                if self.show_chunk_borders { "on" } else { "off" },
            );
        }
        format!(
            "fps {:.0} | chunks {} | verts {} | quads {} | tris {} | borders {}",
            self.fps,
            self.stats.chunks,
            self.stats.vertices,
            self.stats.quads,
            self.stats.triangles,
            if self.show_chunk_borders { "on" } else { "off" },
        )
    }
}

/// Rolling FPS counter: counts frames and recomputes the rate at most
/// every `window`.
#[derive(Debug, Clone)]
pub struct FpsCounter {
    window: Duration,
    frames: u32,
    started: Instant,
    pub fps: f32,
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl FpsCounter {
    pub fn new() -> Self {
        Self {
            window: Duration::from_millis(500),
            frames: 0,
            started: Instant::now(),
            fps: 0.0,
        }
    }

    /// Record one rendered frame at `now`; returns the current fps.
    pub fn tick(&mut self, now: Instant) -> f32 {
        self.frames += 1;
        let elapsed = now.duration_since(self.started);
        if elapsed >= self.window {
            self.fps = self.frames as f32 / elapsed.as_secs_f32();
            self.frames = 0;
            self.started = now;
        }
        self.fps
    }
}

/// Frame pacing (plan Phase 2): the game is turn-based, so render only
/// when something changed or the camera moved. The bin asks `should_render`
/// and calls `rendered` after each presented frame.
#[derive(Debug, Clone, Default)]
pub struct FramePacer {
    dirty: bool,
    camera_dirty: bool,
}

impl FramePacer {
    pub fn new() -> Self {
        Self {
            dirty: true, // first frame always renders
            camera_dirty: false,
        }
    }
    pub fn mark_world_dirty(&mut self) {
        self.dirty = true;
    }
    pub fn mark_camera_dirty(&mut self) {
        self.camera_dirty = true;
    }
    pub fn should_render(&self) -> bool {
        self.dirty || self.camera_dirty
    }
    pub fn rendered(&mut self) {
        self.dirty = false;
        self.camera_dirty = false;
    }
}

/// What a resize event should do (gate 2.11). Zero-size (minimized)
/// surfaces must never be reconfigured or rendered to; an unchanged size
/// needs no work; a real change reconfigures the surface and recreates
/// the depth buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeAction {
    Ignore,
    Reconfigure { width: u32, height: u32 },
}

pub fn resize_action(current: (u32, u32), new: (u32, u32)) -> ResizeAction {
    if new.0 == 0 || new.1 == 0 {
        return ResizeAction::Ignore;
    }
    if new == current {
        return ResizeAction::Ignore;
    }
    ResizeAction::Reconfigure {
        width: new.0,
        height: new.1,
    }
}

/// Whether a frame may be presented at this size (false when minimized).
pub fn renderable(size: (u32, u32)) -> bool {
    size.0 > 0 && size.1 > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression (the Launch-Sim screenshot's title): an RT
    /// frame has no mesh stats, so the summary must report
    /// the traced voxel count — never the zeroed raster
    /// counters that claimed an empty world.
    #[test]
    fn overlay_summary_reports_rt_voxels() {
        let mut o = DebugOverlay::new();
        o.stats.chunks = 4;
        o.rt_voxels = Some(1726);
        let s = o.summary();
        assert!(s.contains("chunks 4"), "{s}");
        assert!(s.contains("rt voxels 1726"), "{s}");
        assert!(!s.contains("quads 0"), "{s}");
    }

    #[test]
    fn overlay_summary_contains_stats() {
        let mut o = DebugOverlay::new();
        o.stats = MeshStats {
            chunks: 3,
            vertices: 40,
            quads: 10,
            triangles: 20,
        };
        o.fps = 59.6;
        let s = o.summary();
        assert!(s.contains("chunks 3"), "{s}");
        assert!(s.contains("quads 10"), "{s}");
        assert!(s.contains("borders off"), "{s}");
        o.show_chunk_borders = true;
        assert!(o.summary().contains("borders on"));
    }

    #[test]
    fn fps_counter_reports_plausible_rate() {
        let mut c = FpsCounter::new();
        let t0 = Instant::now();
        let mut fps = 0.0;
        // 30 frames spread over ~600ms of synthetic time.
        for i in 1..=30 {
            fps = c.tick(t0 + Duration::from_millis(20 * i));
        }
        assert!(fps > 30.0 && fps < 70.0, "fps {fps}");
    }

    #[test]
    fn pacer_renders_only_on_change() {
        let mut p = FramePacer::new();
        assert!(p.should_render(), "first frame");
        p.rendered();
        assert!(!p.should_render(), "idle: no render");
        p.mark_camera_dirty();
        assert!(p.should_render());
        p.rendered();
        assert!(!p.should_render());
        p.mark_world_dirty();
        assert!(p.should_render());
    }

    #[test]
    fn test_2_11_resize_decisions() {
        // Minimize / zero size: ignore, never reconfigure.
        assert_eq!(resize_action((800, 600), (0, 0)), ResizeAction::Ignore);
        assert_eq!(resize_action((800, 600), (0, 600)), ResizeAction::Ignore);
        assert_eq!(resize_action((800, 600), (800, 0)), ResizeAction::Ignore);
        // Unchanged: ignore.
        assert_eq!(resize_action((800, 600), (800, 600)), ResizeAction::Ignore);
        // Real change: reconfigure (depth buffer recreated by the bin).
        assert_eq!(
            resize_action((800, 600), (1024, 768)),
            ResizeAction::Reconfigure {
                width: 1024,
                height: 768
            }
        );
        assert!(!renderable((0, 0)));
        assert!(renderable((1, 1)));
    }
}
