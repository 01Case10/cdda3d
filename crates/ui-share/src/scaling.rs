//! Resize / DPI logic (gate 4.6) as pure functions.
//!
//! Three pixel spaces must not be conflated:
//! - *logical window pixels*: what the windowing system reports for
//!   layout (winit logical size).
//! - *physical window pixels*: logical × DPI scale factor — the
//!   actual framebuffer size the engine renders into.
//! - *display-buffer pixels*: the game's UI buffer size. The game
//!   renders its terminal into a buffer whose size follows the
//!   physical window (the SDL backend's `SCALING_FACTOR` folds the
//!   DPI in), so one display-buffer pixel maps to a (possibly
//!   fractional) number of physical pixels; mouse coordinates must
//!   be converted, never passed through raw.

use proto::ResizeT;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size2 { pub w: u32, pub h: u32 }

/// Physical pixels from logical size at a DPI scale factor.
pub fn physical_size(logical: Size2, dpi_scale: f32) -> Size2 {
    let s = if dpi_scale.is_finite() && dpi_scale > 0.0 { dpi_scale } else { 1.0 };
    Size2 {
        w: ((logical.w as f32 * s).round() as u32).max(1),
        h: ((logical.h as f32 * s).round() as u32).max(1),
    }
}
pub fn logical_size(physical: Size2, dpi_scale: f32) -> Size2 {
    let s = if dpi_scale.is_finite() && dpi_scale > 0.0 { dpi_scale } else { 1.0 };
    Size2 {
        w: ((physical.w as f32 / s).round() as u32).max(1),
        h: ((physical.h as f32 / s).round() as u32).max(1),
    }
}

/// The `Resize` message the engine sends the game on a window change:
/// display-buffer (physical) size plus the DPI scale.
pub fn resize_message(logical: Size2, dpi_scale: f32) -> ResizeT {
    let p = physical_size(logical, dpi_scale);
    let mut r = ResizeT::default();
    r.width = p.w; r.height = p.h; r.dpi = dpi_scale;
    r
}

/// Convert a mouse position in physical window pixels to
/// display-buffer pixels. `window` and `display` are the two sizes;
/// the mapping is proportional (the UI quad fills the window).
pub fn window_to_display(mx: f32, my: f32, window: Size2, display: Size2) -> (i32, i32) {
    if window.w == 0 || window.h == 0 { return (0, 0); }
    let x = (mx * display.w as f32 / window.w as f32).floor() as i32;
    let y = (my * display.h as f32 / window.h as f32).floor() as i32;
    (x.clamp(0, display.w.saturating_sub(1) as i32), y.clamp(0, display.h.saturating_sub(1) as i32))
}
pub fn display_to_window(dx: i32, dy: i32, window: Size2, display: Size2) -> (f32, f32) {
    if display.w == 0 || display.h == 0 { return (0.0, 0.0); }
    (dx as f32 * window.w as f32 / display.w as f32, dy as f32 * window.h as f32 / display.h as f32)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalingMode {
    /// Stretch to fill, nearest-neighbor sampling.
    StretchNearest,
    /// Largest integer scale that fits, centered (letterboxed).
    IntegerScale,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub x: f32, pub y: f32, pub w: f32, pub h: f32,
    pub scale_x: f32, pub scale_y: f32,
}

/// Where the UI quad lands in the window for a given mode.
pub fn compute_viewport(window: Size2, ui: Size2, mode: ScalingMode) -> Viewport {
    if window.w == 0 || window.h == 0 || ui.w == 0 || ui.h == 0 {
        return Viewport { x: 0.0, y: 0.0, w: 0.0, h: 0.0, scale_x: 0.0, scale_y: 0.0 };
    }
    match mode {
        ScalingMode::StretchNearest => Viewport {
            x: 0.0, y: 0.0, w: window.w as f32, h: window.h as f32,
            scale_x: window.w as f32 / ui.w as f32, scale_y: window.h as f32 / ui.h as f32,
        },
        ScalingMode::IntegerScale => {
            let k = ((window.w / ui.w).min(window.h / ui.h)).max(1);
            // If the window is smaller than the UI, fall back to fit-stretch at scale <1 handled by k=... use fractional fit.
            if window.w < ui.w || window.h < ui.h {
                let s = (window.w as f32 / ui.w as f32).min(window.h as f32 / ui.h as f32);
                let (w, h) = (ui.w as f32 * s, ui.h as f32 * s);
                return Viewport { x: (window.w as f32 - w) / 2.0, y: (window.h as f32 - h) / 2.0, w, h, scale_x: s, scale_y: s };
            }
            let (w, h) = ((ui.w * k) as f32, (ui.h * k) as f32);
            Viewport { x: (window.w as f32 - w) / 2.0, y: (window.h as f32 - h) / 2.0, w, h, scale_x: k as f32, scale_y: k as f32 }
        }
    }
}

/// Map a window-pixel mouse position through the viewport back to a
/// UI/display pixel; `None` when the click falls in a letterbox bar.
pub fn window_click_to_ui(mx: f32, my: f32, vp: &Viewport, ui: Size2) -> Option<(i32, i32)> {
    if vp.w <= 0.0 || vp.h <= 0.0 { return None; }
    if mx < vp.x || my < vp.y || mx >= vp.x + vp.w || my >= vp.y + vp.h { return None; }
    let x = ((mx - vp.x) / vp.scale_x).floor() as i32;
    let y = ((my - vp.y) / vp.scale_y).floor() as i32;
    Some((x.clamp(0, ui.w.saturating_sub(1) as i32), y.clamp(0, ui.h.saturating_sub(1) as i32)))
}

/// Decide whether a resize needs action (mirrors the engine's rule:
/// zero-size or unchanged resizes are ignored).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeAction { Ignore, Reconfigure }
pub fn resize_action(old: Size2, new: Size2) -> ResizeAction {
    if new.w == 0 || new.h == 0 || new == old { ResizeAction::Ignore } else { ResizeAction::Reconfigure }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dpi_physical_logical_roundtrip() {
        let logical = Size2 { w: 1280, h: 720 };
        assert_eq!(physical_size(logical, 1.25), Size2 { w: 1600, h: 900 });
        assert_eq!(logical_size(Size2 { w: 1600, h: 900 }, 1.25), logical);
        assert_eq!(physical_size(logical, 1.0), logical);
        assert_eq!(physical_size(logical, 2.0), Size2 { w: 2560, h: 1440 });
    }

    #[test]
    fn mouse_conversion_at_dpi() {
        // Window physical 1600x900, display buffer 1600x900: identity.
        let win = Size2 { w: 1600, h: 900 };
        assert_eq!(window_to_display(800.0, 450.0, win, win), (800, 450));
        // Display buffer half the window (scaling factor 2): halved.
        let disp = Size2 { w: 800, h: 450 };
        assert_eq!(window_to_display(800.0, 450.0, win, disp), (400, 225));
        let (wx, wy) = display_to_window(400, 225, win, disp);
        assert!((wx - 800.0).abs() < 1.0 && (wy - 450.0).abs() < 1.0);
    }

    #[test]
    fn integer_scale_letterboxes() {
        let vp = compute_viewport(Size2 { w: 1000, h: 700 }, Size2 { w: 320, h: 200 }, ScalingMode::IntegerScale);
        assert_eq!((vp.scale_x, vp.scale_y), (3.0, 3.0));
        assert_eq!((vp.w, vp.h), (960.0, 600.0));
        assert_eq!(window_click_to_ui(vp.x + 1.0, vp.y + 1.0, &vp, Size2 { w: 320, h: 200 }), Some((0, 0)));
        assert_eq!(window_click_to_ui(0.0, 0.0, &vp, Size2 { w: 320, h: 200 }), None);
    }

    #[test]
    fn resize_message_carries_physical_and_dpi() {
        let r = resize_message(Size2 { w: 1024, h: 768 }, 1.5);
        assert_eq!((r.width, r.height), (1536, 1152));
        assert_eq!(r.dpi, 1.5);
    }
}
