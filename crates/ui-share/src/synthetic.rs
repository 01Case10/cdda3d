//! Synthetic golden UI frames (gate 4.1 at harness level) and the
//! image-diff helper.
//!
//! The real goldens require the fork running in-game (blocked on
//! 0.1/0.2). What this module proves at harness level is the whole
//! pipeline around them: deterministic CPU-drawn frames in the
//! shapes of the real screens (main menu, inventory, crafting,
//! sidebar gameplay, message log) are written into the shared store,
//! read back, masked, composited, and diffed. Drawing is block
//! graphics — filled rects, 1px borders, and bar glyphs standing in
//! for text lines — because no font rasterizer is in scope; the
//! geometry (panel positions, terrain hole, list rows) is what the
//! compositing gates exercise.

use crate::frame::TerrainRect;

/// A tiny deterministic RGBA canvas.
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}
impl Canvas {
    pub fn new(width: u32, height: u32, bg: [u8; 4]) -> Self {
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        for i in 0..(width * height) as usize { pixels[i * 4..i * 4 + 4].copy_from_slice(&bg); }
        Self { width, height, pixels }
    }
    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: [u8; 4]) {
        for yy in y.max(0)..(y + h).min(self.height as i32) {
            for xx in x.max(0)..(x + w).min(self.width as i32) {
                let o = ((yy as u32 * self.width + xx as u32) * 4) as usize;
                self.pixels[o..o + 4].copy_from_slice(&c);
            }
        }
    }
    pub fn border(&mut self, x: i32, y: i32, w: i32, h: i32, c: [u8; 4]) {
        self.fill_rect(x, y, w, 1, c);
        self.fill_rect(x, y + h - 1, w, 1, c);
        self.fill_rect(x, y, 1, h, c);
        self.fill_rect(x + w - 1, y, 1, h, c);
    }
    /// A fake text line: a row of blocks whose lengths vary by seed,
    /// so different lines diff against each other.
    pub fn text_line(&mut self, x: i32, y: i32, words: u32, seed: u32, c: [u8; 4]) {
        let mut cx = x;
        let mut s = seed.wrapping_mul(2654435761).wrapping_add(97);
        for _ in 0..words {
            s = s.wrapping_mul(1664525).wrapping_add(1013904223);
            let len = 12 + ((s >> 16) % 40) as i32;
            self.fill_rect(cx, y, len, 5, c);
            cx += len + 8;
            if cx > self.width as i32 - 20 { break; }
        }
    }
    pub fn to_image(&self) -> image::RgbaImage {
        image::RgbaImage::from_raw(self.width, self.height, self.pixels.clone()).expect("canvas size")
    }
}

pub const PANEL_BG: [u8; 4] = [24, 24, 32, 255];
pub const PANEL_FG: [u8; 4] = [210, 210, 200, 255];
pub const ACCENT: [u8; 4] = [90, 180, 255, 255];
pub const DIM: [u8; 4] = [110, 110, 120, 255];

/// Terrain rect used by the synthetic gameplay frame: left 2/3.
pub fn synthetic_terrain_rect(w: u32, h: u32) -> TerrainRect {
    TerrainRect { x: 0, y: 0, w: (w * 2 / 3) as i32, h: h as i32 }
}

pub fn draw_main_menu(w: u32, h: u32) -> Canvas {
    let mut c = Canvas::new(w, h, PANEL_BG);
    let (bw, bh) = ((w / 2) as i32, (h / 2) as i32);
    let (bx, by) = ((w / 4) as i32, (h / 4) as i32);
    c.border(bx, by, bw, bh, ACCENT);
    c.text_line(bx + 24, by + 24, 2, 1, ACCENT);
    for i in 0..5 {
        let col = if i == 0 { ACCENT } else { PANEL_FG };
        c.text_line(bx + 32, by + 70 + i * 28, 2, (10 + i) as u32, col);
    }
    c
}

pub fn draw_inventory(w: u32, h: u32) -> Canvas {
    let mut c = Canvas::new(w, h, PANEL_BG);
    c.border(8, 8, w as i32 - 16, h as i32 - 16, PANEL_FG);
    c.text_line(24, 20, 3, 42, ACCENT);
    for i in 0..12 {
        let y = 56 + i * 22;
        if i == 3 { c.fill_rect(16, y - 4, w as i32 - 32, 16, [50, 70, 110, 255]); }
        c.text_line(28, y, 3, (100 + i) as u32, if i == 3 { ACCENT } else { PANEL_FG });
        c.text_line(w as i32 - 140, y, 1, (200 + i) as u32, DIM);
    }
    c
}

pub fn draw_crafting(w: u32, h: u32) -> Canvas {
    let mut c = Canvas::new(w, h, PANEL_BG);
    c.border(8, 8, (w / 3) as i32, h as i32 - 16, PANEL_FG);
    c.border((w / 3) as i32 + 16, 8, (w * 2 / 3) as i32 - 24, h as i32 - 16, PANEL_FG);
    for i in 0..8 { c.text_line(24, 32 + i * 24, 2, (300 + i) as u32, PANEL_FG); }
    for i in 0..6 { c.text_line((w / 3) as i32 + 36, 32 + i * 24, 4, (400 + i) as u32, DIM); }
    c
}

/// Gameplay frame: sidebar panels on the right, message log at the
/// bottom of the sidebar, and the terrain rect left *unpainted*
/// (alpha 0) exactly as Route B capture delivers it — the game
/// suppresses the map window, everything else is opaque.
pub fn draw_gameplay_sidebar(w: u32, h: u32) -> Canvas {
    let tr = synthetic_terrain_rect(w, h);
    let mut c = Canvas::new(w, h, [0, 0, 0, 0]);
    let sx = tr.w;
    c.fill_rect(sx, 0, w as i32 - sx, h as i32, PANEL_BG);
    c.border(sx + 6, 6, w as i32 - sx - 12, (h / 3) as i32, PANEL_FG);
    for i in 0..4 { c.text_line(sx + 18, 20 + i * 22, 2, (500 + i) as u32, PANEL_FG); }
    // Message log panel (bottom of the sidebar in this layout).
    let ly = (h / 3) as i32 + 16;
    c.border(sx + 6, ly, w as i32 - sx - 12, h as i32 - ly - 6, DIM);
    for i in 0..6 { c.text_line(sx + 18, ly + 14 + i * 20, 3, (600 + i) as u32, DIM); }
    c
}

pub fn draw_character_creation(w: u32, h: u32) -> Canvas {
    let mut c = Canvas::new(w, h, PANEL_BG);
    c.text_line(32, 24, 3, 700, ACCENT);
    for col in 0..2 {
        for i in 0..7 {
            c.text_line(32 + col * (w / 2) as i32, 70 + i * 26, 2, (710 + col * 20 + i) as u32, PANEL_FG);
        }
    }
    c
}

pub fn all_synthetic_frames(w: u32, h: u32) -> Vec<(&'static str, Canvas)> {
    vec![
        ("main_menu", draw_main_menu(w, h)),
        ("character_creation", draw_character_creation(w, h)),
        ("inventory", draw_inventory(w, h)),
        ("crafting", draw_crafting(w, h)),
        ("gameplay_sidebar", draw_gameplay_sidebar(w, h)),
    ]
}

// --- image diff --------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DiffStats {
    pub total_pixels: usize,
    pub mismatched_pixels: usize,
    pub max_channel_delta: u8,
    pub mean_abs_delta: f64,
}
impl DiffStats {
    pub fn mismatch_fraction(&self) -> f64 {
        if self.total_pixels == 0 { 0.0 } else { self.mismatched_pixels as f64 / self.total_pixels as f64 }
    }
    /// Pass = no pixel exceeds `per_channel_tolerance` beyond the
    /// allowed mismatch budget (fraction of pixels).
    pub fn passes(&self, budget_fraction: f64) -> bool {
        self.mismatch_fraction() <= budget_fraction
    }
}

/// Compare two tight RGBA buffers; a pixel mismatches when any
/// channel differs by more than `per_channel_tolerance`.
pub fn image_diff(a: &[u8], b: &[u8], per_channel_tolerance: u8) -> DiffStats {
    let n = a.len().min(b.len()) / 4;
    let mut mismatched = 0usize;
    let mut max_delta = 0u8;
    let mut sum: u64 = 0;
    for i in 0..n {
        let mut bad = false;
        for ch in 0..4 {
            let d = (a[i * 4 + ch] as i16 - b[i * 4 + ch] as i16).unsigned_abs() as u8;
            max_delta = max_delta.max(d);
            sum += d as u64;
            if d > per_channel_tolerance { bad = true; }
        }
        if bad { mismatched += 1; }
    }
    DiffStats {
        total_pixels: n,
        mismatched_pixels: mismatched,
        max_channel_delta: max_delta,
        mean_abs_delta: if n == 0 { 0.0 } else { sum as f64 / (n * 4) as f64 },
    }
}

pub fn save_png(path: &std::path::Path, pixels: &[u8], width: u32, height: u32) -> std::io::Result<()> {
    let img = image::RgbaImage::from_raw(width, height, pixels.to_vec())
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "pixel buffer size mismatch"))?;
    if let Some(p) = path.parent() { std::fs::create_dir_all(p)?; }
    img.save(path).map_err(std::io::Error::other)
}
