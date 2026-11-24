//! UI state machine + real-font rendering (slice S4, plan §3.3
//! — Joey's decision: real rendered text, not the synthetic
//! bar-glyph stand-ins).
//!
//! The sim renders its UI the way the fork does: a
//! catacurses-style character-cell grid (per-cell char + fg/bg +
//! bold/underline), rasterised with a real font into an RGBA
//! frame, written into the shared-memory store in the
//! `crates/ui-share` format (double-buffered, header + slot
//! checksums — `UiShmStore` is the writer, exactly the contract
//! the engine reads). Pixels never travel on the socket; every
//! publish is announced by one `UiFrame` message carrying the
//! buffer index, seq, dirty rect, mode, and terrain rect.
//!
//! Font (S4 decision, recorded in docs/test-sim-engine.md §3.3):
//! **Liberation Mono Regular**, SIL Open Font License 1.1 (the
//! §3.3 documented fallback — CDDA's ensured fallback is
//! Unifont, whose licence/embedding is awkward), bundled at
//! `crates/sim/assets/LiberationMono-Regular.ttf` (licence text
//! alongside it). SHA-256 of the bundled file:
//! 395fa5ab8d40c8eba390ced528744ea75a7f69aabf3e68b6f925ca0e39a27370.
//! UI goldens are font-version-sensitive (§3.3): the FNV-1a of
//! the exact bytes is pinned in `FONT_FNV1A` and asserted by a
//! unit test, so a silent font swap fails loudly.
//!
//! Rasteriser: `ab_glyph` (§3.3 names fontdue / ab_glyph;
//! ab_glyph 0.2 is already in the workspace lock and the local
//! registry cache — fontdue is not fetchable here). Glyphs are
//! rasterised once into a cache; bold is a 1px double-strike
//! (the bundle ships the regular face only). Wide chars are
//! deferred (§3.3): every char occupies one cell.
//!
//! States (§3.3): Gameplay (mode MapInput, sidebar frame,
//! terrain rect = `synthetic_terrain_rect`), Inventory,
//! Crafting, LookAround, MainMenu (all mode UiInput,
//! full-frame). Transitions: `i` → Inventory, `&` → Crafting,
//! `;` → LookAround, Esc/`q` → back to Gameplay. MainMenu is an
//! entry state only (scene `ui_start` key / sim-server
//! `--ui-start`): it is NOT on Esc-from-Gameplay (T2 pins Esc
//! semantics), and its entries are display-only.

use std::collections::HashMap;
use std::path::PathBuf;

use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use proto::{UiFrameT, UiMode};
use ui_share::frame::frame_notification;
use ui_share::store::{DirtyRect, StoreError, UiShmStore};
use ui_share::synthetic::synthetic_terrain_rect;
use ui_share::TerrainRect;

pub use ui_share::input::{
    KEY_DOWN, KEY_ESCAPE, KEY_LEFT, KEY_PAGE_DOWN, KEY_PAGE_UP, KEY_RIGHT, KEY_UP,
};

/// The bundled font file's bytes (see module docs for the
/// licence + SHA-256).
pub const FONT_BYTES: &[u8] = include_bytes!("../assets/LiberationMono-Regular.ttf");

/// FNV-1a 64 of `FONT_BYTES` — the golden pin (§3.3: goldens
/// pin the bundled font file's hash).
pub const FONT_FNV1A: u64 = 0x0b0d_852d_c58d_8cc0;

/// Font size in pixels the cell grid is derived from.
pub const FONT_PX: f32 = 16.0;

/// In-process default UI size (the server replaces it with the
/// HelloAck screen size; a Resize replaces it again).
pub const DEFAULT_UI_SIZE: (u32, u32) = (1280, 720);

// Palette: the synthetic frames' colours (ui_share::synthetic),
// so the real-font frames read as the same screens.
pub const COL_BG: [u8; 4] = [24, 24, 32, 255];
pub const COL_FG: [u8; 4] = [210, 210, 200, 255];
pub const COL_ACCENT: [u8; 4] = [90, 180, 255, 255];
pub const COL_DIM: [u8; 4] = [110, 110, 120, 255];
pub const COL_HILITE_BG: [u8; 4] = [50, 70, 110, 255];
pub const COL_CLEAR: [u8; 4] = [0, 0, 0, 0];
pub const COL_GREEN: [u8; 4] = [120, 220, 120, 255];
pub const COL_RED: [u8; 4] = [230, 110, 110, 255];

/// Crafting screen contents: a fixed recipe list (the screen
/// opens and navigates; crafting itself is a §3.2 non-goal).
pub const RECIPES: &[&str] = &[
    "bandage",
    "torch",
    "pointy stick",
    "makeshift knife",
    "rag",
    "wooden spear",
    "fire drill",
    "sewing kit (improvised)",
];

/// Main-menu entries (display-only in the sim).
pub const MENU_ENTRIES: &[&str] = &["New Game", "Load Game", "Options", "Help", "Quit"];

/// FNV-1a 64 (same constants as `ui_share::store::frame_checksum`).
pub fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

// ---------------------------------------------------------------------
// Cell grid
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: [u8; 4],
    pub bg: [u8; 4],
    pub bold: bool,
    pub underline: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self { ch: ' ', fg: COL_FG, bg: COL_CLEAR, bold: false, underline: false }
    }
}

/// A catacurses-style character-cell grid: the UI's source of
/// truth. Frames are rasterised from it; the dirty rect is the
/// pixel bbox of cells that changed since the last publish;
/// gates assert on its text directly (the honest-text rule).
#[derive(Debug, Clone, PartialEq)]
pub struct CellGrid {
    pub cols: usize,
    pub rows: usize,
    pub cells: Vec<Cell>,
}

impl CellGrid {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self { cols, rows, cells: vec![Cell::default(); cols * rows] }
    }

    pub fn clear(&mut self, bg: [u8; 4]) {
        for c in &mut self.cells {
            *c = Cell { bg, ..Cell::default() };
        }
    }

    pub fn set(&mut self, x: usize, y: usize, cell: Cell) {
        if x < self.cols && y < self.rows {
            self.cells[y * self.cols + x] = cell;
        }
    }

    pub fn put(&mut self, x: usize, y: usize, ch: char, fg: [u8; 4], bg: [u8; 4]) {
        self.set(x, y, Cell { ch, fg, bg, bold: false, underline: false });
    }

    /// Write a string starting at (x, y); returns the next free
    /// column. Chars past the row edge are clipped (wide chars
    /// deferred — one char, one cell, §3.3).
    pub fn text(&mut self, x: usize, y: usize, s: &str, fg: [u8; 4], bg: [u8; 4]) -> usize {
        let mut cx = x;
        for ch in s.chars() {
            if cx >= self.cols {
                break;
            }
            self.put(cx, y, ch, fg, bg);
            cx += 1;
        }
        cx
    }

    pub fn text_bold(&mut self, x: usize, y: usize, s: &str, fg: [u8; 4], bg: [u8; 4]) -> usize {
        let mut cx = x;
        for ch in s.chars() {
            if cx >= self.cols {
                break;
            }
            self.set(cx, y, Cell { ch, fg, bg, bold: true, underline: false });
            cx += 1;
        }
        cx
    }

    pub fn fill_bg(&mut self, x: usize, y: usize, w: usize, h: usize, bg: [u8; 4]) {
        for yy in y..(y + h).min(self.rows) {
            for xx in x..(x + w).min(self.cols) {
                let c = &mut self.cells[yy * self.cols + xx];
                c.bg = bg;
            }
        }
    }

    /// Box-drawing border around the rect (x, y, w, h) in cells.
    pub fn border(&mut self, x: usize, y: usize, w: usize, h: usize, fg: [u8; 4], bg: [u8; 4]) {
        if w < 2 || h < 2 || x + w > self.cols || y + h > self.rows {
            return;
        }
        for xx in x..x + w {
            self.put(xx, y, '─', fg, bg);
            self.put(xx, y + h - 1, '─', fg, bg);
        }
        for yy in y..y + h {
            self.put(x, yy, '│', fg, bg);
            self.put(x + w - 1, yy, '│', fg, bg);
        }
        self.put(x, y, '┌', fg, bg);
        self.put(x + w - 1, y, '┐', fg, bg);
        self.put(x, y + h - 1, '└', fg, bg);
        self.put(x + w - 1, y + h - 1, '┘', fg, bg);
    }

    /// Row strings (trailing spaces trimmed) — the gate view of
    /// the frame's text.
    pub fn row_strings(&self) -> Vec<String> {
        (0..self.rows)
            .map(|y| {
                let row: String = self.cells[y * self.cols..(y + 1) * self.cols]
                    .iter()
                    .map(|c| c.ch)
                    .collect();
                row.trim_end().to_string()
            })
            .collect()
    }
}

// ---------------------------------------------------------------------
// Font + glyph cache
// ---------------------------------------------------------------------

struct GlyphImg {
    /// Coverage bitmap, row-major, `w * h` bytes (0..=255).
    cov: Vec<u8>,
    w: i32,
    h: i32,
    /// px_bounds().min relative to the glyph origin (pen at the
    /// cell's left edge, baseline at cell_top + ascent).
    min_x: i32,
    min_y: i32,
}

pub struct FontSet {
    font: FontRef<'static>,
    scale: PxScale,
    /// Cell advance in px (the font is monospace: one advance
    /// for every glyph).
    pub cell_w: u32,
    /// Cell height in px (ascent - descent + line gap).
    pub cell_h: u32,
    ascent_px: f32,
    // Arc (not Rc) so `Ui` is Send: the torn-frame gate races a
    // writer thread against a reader thread, mirroring the real
    // game/engine split.
    cache: HashMap<(char, bool), std::sync::Arc<GlyphImg>>,
}

impl FontSet {
    pub fn bundled() -> Self {
        let font = FontRef::try_from_slice(FONT_BYTES).expect("bundled font parses");
        let scale = PxScale::from(FONT_PX);
        let scaled = font.as_scaled(scale);
        let cell_w = scaled.h_advance(scaled.glyph_id('M')).ceil().max(1.0) as u32;
        let line = scaled.ascent() - scaled.descent() + scaled.line_gap();
        let cell_h = line.ceil().max(1.0) as u32;
        Self {
            scale,
            ascent_px: scaled.ascent(),
            font,
            cell_w,
            cell_h,
            cache: HashMap::new(),
        }
    }

    fn glyph(&mut self, ch: char, bold: bool) -> std::sync::Arc<GlyphImg> {
        if let Some(g) = self.cache.get(&(ch, bold)) {
            return g.clone();
        }
        let mut img = self.rasterise(ch);
        // Bold = double-strike: widen the coverage by OR-ing the
        // bitmap with itself shifted 1px right (done once, here,
        // so the blitter stays single-pass).
        if bold && img.w > 0 {
            let orig = img.cov.clone();
            for y in 0..img.h {
                for x in 0..img.w {
                    let i = (y * img.w + x) as usize;
                    let left = if x > 0 { orig[(y * img.w + x - 1) as usize] } else { 0 };
                    img.cov[i] = img.cov[i].max(left);
                }
            }
        }
        let img = std::sync::Arc::new(img);
        self.cache.insert((ch, bold), img.clone());
        img
    }

    fn rasterise(&self, ch: char) -> GlyphImg {
        let glyph: ab_glyph::Glyph = self
            .font
            .glyph_id(ch)
            .with_scale_and_position(self.scale, ab_glyph::point(0.0, 0.0));
        let Some(outlined) = self.font.outline_glyph(glyph) else {
            return GlyphImg { cov: Vec::new(), w: 0, h: 0, min_x: 0, min_y: 0 };
        };
        let bounds = outlined.px_bounds();
        let w = (bounds.max.x - bounds.min.x).ceil() as i32;
        let h = (bounds.max.y - bounds.min.y).ceil() as i32;
        let mut cov = vec![0u8; (w.max(0) * h.max(0)) as usize];
        outlined.draw(|x, y, c| {
            let i = (y as i32 * w + x as i32) as usize;
            if i < cov.len() {
                cov[i] = (c.clamp(0.0, 1.0) * 255.0).round() as u8;
            }
        });
        GlyphImg { cov, w, h, min_x: bounds.min.x as i32, min_y: bounds.min.y as i32 }
    }

    /// Rasterise a grid into a tightly-packed RGBA buffer of
    /// `width * height` px (the grid's cell size times its
    /// cols/rows, clamped to the pixel size).
    fn rasterise_grid(&mut self, grid: &CellGrid, width: u32, height: u32, out: &mut Vec<u8>) {
        out.clear();
        out.resize((width * height * 4) as usize, 0);
        let (cw, chh) = (self.cell_w as i32, self.cell_h as i32);
        // Pass 1: cell backgrounds.
        for gy in 0..grid.rows {
            for gx in 0..grid.cols {
                let cell = grid.cells[gy * grid.cols + gx];
                if cell.bg[3] == 0 {
                    continue;
                }
                let (x0, y0) = (gx as i32 * cw, gy as i32 * chh);
                for yy in y0..(y0 + chh).min(height as i32) {
                    for xx in x0..(x0 + cw).min(width as i32) {
                        let o = ((yy as u32 * width + xx as u32) * 4) as usize;
                        out[o..o + 4].copy_from_slice(&cell.bg);
                    }
                }
            }
        }
        // Pass 2: glyphs (coverage-blended over the bg) +
        // underlines.
        for gy in 0..grid.rows {
            for gx in 0..grid.cols {
                let cell = grid.cells[gy * grid.cols + gx];
                if cell.ch == ' ' && !cell.underline {
                    continue;
                }
                let (x0, y0) = (gx as i32 * cw, gy as i32 * chh);
                if cell.underline {
                    let uy = y0 + chh - 2;
                    if uy >= 0 && uy < height as i32 {
                        for xx in x0..(x0 + cw).min(width as i32) {
                            let o = ((uy as u32 * width + xx as u32) * 4) as usize;
                            out[o..o + 4].copy_from_slice(&cell.fg);
                        }
                    }
                }
                if cell.ch == ' ' {
                    continue;
                }
                let img = self.glyph(cell.ch, cell.bold);
                let base_x = x0 + img.min_x;
                let base_y = y0 + self.ascent_px.round() as i32 + img.min_y;
                for row in 0..img.h {
                    let yy = base_y + row;
                    if yy < 0 || yy >= height as i32 {
                        continue;
                    }
                    for col in 0..img.w {
                        let xx = base_x + col;
                        if xx < 0 || xx >= width as i32 {
                            continue;
                        }
                        let cov = img.cov[(row * img.w + col) as usize] as u32;
                        if cov == 0 {
                            continue;
                        }
                        let o = ((yy as u32 * width + xx as u32) * 4) as usize;
                        for ch in 0..3 {
                            let fg = cell.fg[ch] as u32;
                            let bg = out[o + ch] as u32;
                            out[o + ch] = ((fg * cov + bg * (255 - cov)) / 255) as u8;
                        }
                        out[o + 3] = 255;
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------
// State machine
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiState {
    Gameplay,
    Inventory,
    Crafting,
    LookAround,
    MainMenu,
}

impl UiState {
    pub fn mode(&self) -> UiMode {
        match self {
            UiState::Gameplay => UiMode::MapInput,
            _ => UiMode::UiInput,
        }
    }

    /// Parse a state name (scene `ui_start` key / `--ui-start`).
    pub fn parse(name: &str) -> Option<UiState> {
        match name {
            "gameplay" => Some(UiState::Gameplay),
            "inventory" => Some(UiState::Inventory),
            "crafting" => Some(UiState::Crafting),
            "look" | "lookaround" | "look_around" => Some(UiState::LookAround),
            "main-menu" | "main_menu" | "menu" => Some(UiState::MainMenu),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            UiState::Gameplay => "gameplay",
            UiState::Inventory => "inventory",
            UiState::Crafting => "crafting",
            UiState::LookAround => "look",
            UiState::MainMenu => "main-menu",
        }
    }
}

/// Where the UI store lives. In-process gates use memory; the
/// server uses a file (the engine opens the same path —
/// docs/phase4-ui.md's `--ui-shm <path>` convention).
#[derive(Debug, Clone)]
pub enum StoreKind {
    Memory,
    File(PathBuf),
}

/// Everything the renderer needs from the world, as plain data
/// (the world assembles it; ui.rs never touches world state).
#[derive(Debug, Clone, Default)]
pub struct UiView {
    pub scene_name: String,
    pub player_pos: Option<[i32; 3]>,
    pub player_hp: Option<u8>,
    /// "Day 1  12:00" — formatted by the world from its clock.
    pub clock: String,
    pub weather: String,
    /// Carried items as (display name, count), in list order.
    pub carried: Vec<(String, u16)>,
    /// Message-log lines, oldest first.
    pub messages: Vec<String>,
    /// Look-around tile description lines (world-built; only
    /// read in the LookAround state).
    pub look_lines: Vec<String>,
}

/// Layout numbers shared by the renderer and the click
/// hit-test. Cell-quantised from the synthetic frames' pixel
/// geometry: full-frame panels inset ~1 cell (synthetic: 8px),
/// list rows start under the title, gameplay sidebar = the
/// cells right of the terrain rect.
#[derive(Debug, Clone, Copy)]
pub struct Layout {
    pub cols: usize,
    pub rows: usize,
    pub cell_w: u32,
    pub cell_h: u32,
    pub terrain: TerrainRect,
    /// First sidebar column (gameplay): the first cell column
    /// fully right of the terrain rect.
    pub sidebar_col: usize,
    /// Full-frame list geometry: first list row (grid row) and
    /// the panel's inner left column.
    pub list_top_row: usize,
    pub list_left_col: usize,
}

impl Layout {
    pub fn new(width: u32, height: u32, cell_w: u32, cell_h: u32) -> Layout {
        let cols = (width / cell_w.max(1)) as usize;
        let rows = (height / cell_h.max(1)) as usize;
        let terrain = synthetic_terrain_rect(width, height);
        let sidebar_col = ((terrain.w as u32).div_ceil(cell_w.max(1))) as usize;
        Layout {
            cols,
            rows,
            cell_w,
            cell_h,
            terrain,
            sidebar_col: sidebar_col.min(cols),
            list_top_row: 3,
            list_left_col: 2,
        }
    }
}

pub struct Ui {
    state: UiState,
    store: UiShmStore,
    kind: StoreKind,
    width: u32,
    height: u32,
    font: FontSet,
    grid: CellGrid,
    prev_grid: Option<CellGrid>,
    /// List cursor (Inventory / Crafting / MainMenu).
    cursor: usize,
    /// Look-around cursor tile (absolute world pos).
    look_cursor: [i32; 3],
    pixels: Vec<u8>,
}

impl std::fmt::Debug for Ui {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ui")
            .field("state", &self.state)
            .field("size", &(self.width, self.height))
            .field("cursor", &self.cursor)
            .field("look_cursor", &self.look_cursor)
            .finish_non_exhaustive()
    }
}

impl Ui {
    pub fn new_memory(width: u32, height: u32) -> Result<Ui, StoreError> {
        let store = UiShmStore::create_memory(width, height)?;
        Ok(Self::with_store(store, StoreKind::Memory, width, height))
    }

    pub fn new_file(path: PathBuf, width: u32, height: u32) -> Result<Ui, StoreError> {
        let store = UiShmStore::create_file(&path, width, height)?;
        Ok(Self::with_store(store, StoreKind::File(path), width, height))
    }

    fn with_store(store: UiShmStore, kind: StoreKind, width: u32, height: u32) -> Ui {
        let font = FontSet::bundled();
        let layout = Layout::new(width, height, font.cell_w, font.cell_h);
        Ui {
            state: UiState::Gameplay,
            store,
            kind,
            width,
            height,
            grid: CellGrid::new(layout.cols, layout.rows),
            prev_grid: None,
            font,
            cursor: 0,
            look_cursor: [0, 0, 0],
            pixels: Vec::new(),
        }
    }

    pub fn state(&self) -> UiState {
        self.state
    }

    pub fn mode(&self) -> UiMode {
        self.state.mode()
    }

    pub fn store(&self) -> &UiShmStore {
        &self.store
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn layout(&self) -> Layout {
        Layout::new(self.width, self.height, self.font.cell_w, self.font.cell_h)
    }

    pub fn grid(&self) -> &CellGrid {
        &self.grid
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn look_cursor(&self) -> [i32; 3] {
        self.look_cursor
    }

    pub fn set_look_cursor(&mut self, pos: [i32; 3]) {
        self.look_cursor = pos;
    }

    /// Enter a UI state (from Gameplay, on its binding key).
    /// The look cursor starts on the player.
    pub fn open(&mut self, state: UiState, player_pos: Option<[i32; 3]>) {
        self.state = state;
        self.cursor = 0;
        if state == UiState::LookAround {
            self.look_cursor = player_pos.unwrap_or([0, 0, 0]);
        }
    }

    /// Force the state (entry-state selection: scene `ui_start`
    /// / `--ui-start`).
    pub fn set_state(&mut self, state: UiState, player_pos: Option<[i32; 3]>) {
        self.open(state, player_pos);
    }

    fn list_len(&self, view: &UiView) -> usize {
        match self.state {
            UiState::Inventory => view.carried.len(),
            UiState::Crafting => RECIPES.len(),
            UiState::MainMenu => MENU_ENTRIES.len(),
            _ => 0,
        }
    }

    /// Handle one Key press while in a UiInput state. Returns
    /// true when anything visible changed (state, cursor, look
    /// cursor) and a new frame must be published. `look_bounds`
    /// clamps the look cursor (the world's scene extent).
    pub fn apply_ui_key(
        &mut self,
        key_code: i32,
        view: &UiView,
        look_bounds: ([i32; 3], [i32; 3]),
    ) -> bool {
        let len = self.list_len(view);
        match key_code {
            k if k == KEY_ESCAPE => {
                // Esc leaves every UiInput state for Gameplay —
                // except MainMenu, the entry state: there is no
                // game behind it yet, so Esc is a no-op there.
                if self.state == UiState::MainMenu {
                    return false;
                }
                self.state = UiState::Gameplay;
                return true;
            }
            k if k == KEY_UP => {
                return if self.state == UiState::LookAround {
                    self.move_look(0, -1, look_bounds)
                } else {
                    self.move_cursor(-1, len)
                };
            }
            k if k == KEY_DOWN => {
                return if self.state == UiState::LookAround {
                    self.move_look(0, 1, look_bounds)
                } else {
                    self.move_cursor(1, len)
                };
            }
            k if k == KEY_LEFT => {
                return if self.state == UiState::LookAround {
                    self.move_look(-1, 0, look_bounds)
                } else {
                    self.move_cursor(-1, len)
                };
            }
            k if k == KEY_RIGHT => {
                return if self.state == UiState::LookAround {
                    self.move_look(1, 0, look_bounds)
                } else {
                    self.move_cursor(1, len)
                };
            }
            k if k == KEY_PAGE_UP => return self.move_cursor(-10, len),
            k if k == KEY_PAGE_DOWN => return self.move_cursor(10, len),
            _ => {}
        }
        let Some(ch) = char::from_u32(key_code as u32) else {
            return false;
        };
        match ch {
            // `q` backs out like Esc (§3.3) — except in
            // MainMenu, where CDDA binds it to Quit; the sim's
            // menu entries are display-only, so it is a no-op.
            'q' if self.state != UiState::MainMenu => {
                self.state = UiState::Gameplay;
                true
            }
            // Cursor keys follow the world keymap settlement
            // (S6): `s`/`d` are smash/drop in MapInput, so the
            // UI drops those movement aliases too — vi-keys
            // (plus `w`/`a`) everywhere.
            'j' | 'l' if self.state != UiState::LookAround => self.move_cursor(1, len),
            'k' | 'w' | 'h' if self.state != UiState::LookAround => self.move_cursor(-1, len),
            _ if self.state == UiState::LookAround => {
                let (dx, dy) = match ch {
                    'h' | 'a' => (-1, 0),
                    'l' => (1, 0),
                    'k' | 'w' => (0, -1),
                    'j' => (0, 1),
                    'y' => (-1, -1),
                    'u' => (1, -1),
                    'b' => (-1, 1),
                    'n' => (1, 1),
                    _ => return false,
                };
                self.move_look(dx, dy, look_bounds)
            }
            _ => false,
        }
    }

    fn move_cursor(&mut self, delta: i32, len: usize) -> bool {
        if len == 0 {
            return false;
        }
        let next = (self.cursor as i32 + delta).clamp(0, len as i32 - 1) as usize;
        if next == self.cursor {
            return false;
        }
        self.cursor = next;
        true
    }

    fn move_look(&mut self, dx: i32, dy: i32, bounds: ([i32; 3], [i32; 3])) -> bool {
        let next = [
            (self.look_cursor[0] + dx).clamp(bounds.0[0], bounds.1[0]),
            (self.look_cursor[1] + dy).clamp(bounds.0[1], bounds.1[1]),
            self.look_cursor[2],
        ];
        if next == self.look_cursor {
            return false;
        }
        self.look_cursor = next;
        true
    }

    /// Mouse click (UI pixels) while in UiInput: hit-test the
    /// list / menu rows. Returns true when the cursor moved.
    pub fn apply_click(&mut self, x: i32, y: i32, view: &UiView) -> bool {
        let len = self.list_len(view);
        if len == 0 {
            return false;
        }
        let layout = self.layout();
        let (top_row, left_col) = match self.state {
            UiState::MainMenu => {
                let (bx, by, _, _) = menu_box(&layout);
                (by + 4, bx + 2)
            }
            _ => (layout.list_top_row, layout.list_left_col),
        };
        if x < (left_col * layout.cell_w as usize) as i32 || y < (top_row * layout.cell_h as usize) as i32 {
            return false;
        }
        let idx = ((y as usize) / layout.cell_h as usize).saturating_sub(top_row);
        if idx >= len || idx == self.cursor {
            return false;
        }
        self.cursor = idx;
        true
    }

    /// Wheel scroll while in UiInput: move the list cursor by
    /// the wheel delta (lines).
    pub fn apply_wheel(&mut self, delta: i32, view: &UiView) -> bool {
        let len = self.list_len(view);
        if delta == 0 || len == 0 || self.state == UiState::LookAround {
            return false;
        }
        self.move_cursor(delta, len)
    }

    /// Rebuild the store at a new size (Resize): the store's
    /// geometry is fixed at creation, so a resize is a new
    /// store generation at the same location (same path / same
    /// in-process role); the next publish is a full frame. The
    /// state machine (state, cursor, look cursor) survives.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), StoreError> {
        if (width, height) == (self.width, self.height) || width == 0 || height == 0 {
            return Ok(());
        }
        self.store = match &self.kind {
            StoreKind::Memory => UiShmStore::create_memory(width, height)?,
            StoreKind::File(path) => UiShmStore::create_file(path, width, height)?,
        };
        self.width = width;
        self.height = height;
        let layout = self.layout();
        self.grid = CellGrid::new(layout.cols, layout.rows);
        self.prev_grid = None;
        Ok(())
    }

    /// Swap the backing store for a file store at `path` (the
    /// server does this once, after the HelloAck reports the
    /// screen size): same state machine, new store generation
    /// at the given size; the next publish is a full frame.
    pub fn attach_file(&mut self, path: PathBuf, width: u32, height: u32) -> Result<(), StoreError> {
        self.store = UiShmStore::create_file(&path, width, height)?;
        self.kind = StoreKind::File(path);
        self.width = width;
        self.height = height;
        let layout = self.layout();
        self.grid = CellGrid::new(layout.cols, layout.rows);
        self.prev_grid = None;
        Ok(())
    }

    /// Render the current state from `view`, publish into the
    /// store, and return the UiFrame notification for the
    /// socket. The dirty rect is the pixel bbox of the cells
    /// that changed since the last publish (full frame on the
    /// first publish and after a resize).
    pub fn publish(&mut self, view: &UiView) -> Result<UiFrameT, StoreError> {
        self.render(view);
        let dirty = match &self.prev_grid {
            None => DirtyRect::full(self.width, self.height),
            Some(prev) => self.changed_rect(prev),
        };
        self.font
            .rasterise_grid(&self.grid, self.width, self.height, &mut self.pixels);
        let pixels = std::mem::take(&mut self.pixels);
        let seq = self.store.write_frame_tight(&pixels, self.width, self.height, dirty)?;
        self.pixels = pixels;
        debug_assert!(seq > 0);
        let snap = self.store.read_latest()?;
        let terrain = match self.state {
            UiState::Gameplay => Some(self.layout().terrain),
            _ => None,
        };
        self.prev_grid = Some(self.grid.clone());
        Ok(frame_notification(&snap, self.state.mode(), terrain))
    }

    fn changed_rect(&self, prev: &CellGrid) -> DirtyRect {
        let layout = self.layout();
        if prev.cols != self.grid.cols || prev.rows != self.grid.rows {
            return DirtyRect::full(self.width, self.height);
        }
        let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0usize, 0usize);
        for y in 0..self.grid.rows {
            for x in 0..self.grid.cols {
                if prev.cells[y * prev.cols + x] != self.grid.cells[y * self.grid.cols + x] {
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x);
                    y1 = y1.max(y);
                }
            }
        }
        if x0 == usize::MAX {
            return DirtyRect { x: 0, y: 0, w: 0, h: 0 };
        }
        DirtyRect {
            x: (x0 as u32 * layout.cell_w) as i32,
            y: (y0 as u32 * layout.cell_h) as i32,
            w: ((x1 - x0 + 1) as u32 * layout.cell_w) as i32,
            h: ((y1 - y0 + 1) as u32 * layout.cell_h) as i32,
        }
    }

    // -----------------------------------------------------------------
    // Rendering: state -> cell grid. Geometry mirrors the five
    // synthetic frames (same panels, same terrain rect rule);
    // the bar glyphs become real text.
    // -----------------------------------------------------------------

    fn render(&mut self, view: &UiView) {
        let layout = self.layout();
        if self.grid.cols != layout.cols || self.grid.rows != layout.rows {
            self.grid = CellGrid::new(layout.cols, layout.rows);
        }
        match self.state {
            UiState::Gameplay => self.render_gameplay(view, &layout),
            UiState::Inventory => self.render_inventory(view, &layout),
            UiState::Crafting => self.render_crafting(view, &layout),
            UiState::LookAround => self.render_look(view, &layout),
            UiState::MainMenu => self.render_menu(view, &layout),
        }
    }

    fn render_gameplay(&mut self, view: &UiView, layout: &Layout) {
        let g = &mut self.grid;
        g.clear(COL_CLEAR);
        let sb = layout.sidebar_col;
        if sb >= layout.cols {
            return;
        }
        // Sidebar background (the terrain rect stays unpainted,
        // alpha 0 — Route B's shape, as in draw_gameplay_sidebar).
        g.fill_bg(sb, 0, layout.cols - sb, layout.rows, COL_BG);
        let w = layout.cols - sb;
        // Status panel: top third (synthetic: border at h/3).
        let top_h = (layout.rows / 3).max(6);
        g.border(sb, 0, w, top_h, COL_FG, COL_BG);
        let x = sb + 2;
        g.text_bold(x, 1, "Cataclysm: Sim", COL_ACCENT, COL_BG);
        let hp = view.player_hp.map_or(100, |h| h as u32 * 100 / 255);
        let hp_col = if hp > 50 { COL_GREEN } else { COL_RED };
        g.text(x, 2, &format!("HP: {hp}%"), hp_col, COL_BG);
        if let Some(p) = view.player_pos {
            g.text(x, 3, &format!("Pos: ({}, {}, {})", p[0], p[1], p[2]), COL_FG, COL_BG);
        }
        g.text(x, 4, &view.clock, COL_FG, COL_BG);
        g.text(x, 5, &format!("Weather: {}", view.weather), COL_FG, COL_BG);
        g.text(x, 6, "i: inventory  &: craft", COL_DIM, COL_BG);
        g.text(x, 7, ";: look around", COL_DIM, COL_BG);
        // Message log panel: the rest of the sidebar.
        let ly = top_h + 1;
        if ly + 2 < layout.rows {
            g.border(sb, ly, w, layout.rows - ly, COL_DIM, COL_BG);
            let avail = layout.rows - ly - 1;
            let start = view.messages.len().saturating_sub(avail);
            for (i, line) in view.messages[start..].iter().enumerate() {
                g.text(x, ly + 1 + i, line, COL_DIM, COL_BG);
            }
        }
    }

    fn panel_frame(&mut self, layout: &Layout, title: &str) {
        let g = &mut self.grid;
        g.clear(COL_BG);
        if layout.cols >= 2 && layout.rows >= 2 {
            g.border(0, 0, layout.cols, layout.rows, COL_FG, COL_BG);
        }
        g.text_bold(2, 1, title, COL_ACCENT, COL_BG);
    }

    fn render_inventory(&mut self, view: &UiView, layout: &Layout) {
        self.panel_frame(layout, "Inventory");
        let g = &mut self.grid;
        let top = layout.list_top_row;
        if view.carried.is_empty() {
            g.text(layout.list_left_col, top, "(nothing carried)", COL_DIM, COL_BG);
        }
        for (i, (name, count)) in view.carried.iter().enumerate() {
            let y = top + i;
            if y + 1 >= layout.rows {
                break;
            }
            let selected = i == self.cursor;
            let bg = if selected { COL_HILITE_BG } else { COL_BG };
            if selected {
                g.fill_bg(1, y, layout.cols - 2, 1, COL_HILITE_BG);
            }
            let marker = if selected { ">" } else { " " };
            let letter = (b'a' + (i % 26) as u8) as char;
            let mut line = format!("{marker} {letter}) {name}");
            if *count > 1 {
                line.push_str(&format!(" × {count}"));
            }
            let fg = if selected { COL_ACCENT } else { COL_FG };
            g.text(layout.list_left_col, y, &line, fg, bg);
        }
        if layout.rows >= 2 {
            g.text(2, layout.rows - 2, "j/k: move   Esc/q: back", COL_DIM, COL_BG);
        }
    }

    fn render_crafting(&mut self, _view: &UiView, layout: &Layout) {
        self.panel_frame(layout, "Crafting");
        let g = &mut self.grid;
        let split = layout.cols / 3;
        // Left pane: recipe list (synthetic: left third panel).
        if split >= 4 && layout.rows >= 4 {
            g.border(0, 2, split, layout.rows - 3, COL_FG, COL_BG);
        }
        let top = layout.list_top_row + 1;
        for (i, name) in RECIPES.iter().enumerate() {
            let y = top + i;
            if y + 1 >= layout.rows {
                break;
            }
            let selected = i == self.cursor;
            let bg = if selected { COL_HILITE_BG } else { COL_BG };
            if selected {
                g.fill_bg(1, y, split - 2, 1, COL_HILITE_BG);
            }
            let marker = if selected { ">" } else { " " };
            let fg = if selected { COL_ACCENT } else { COL_FG };
            g.text(2, y, &format!("{marker} {name}"), fg, bg);
        }
        // Right pane: the selected recipe, display-only.
        let dx = split + 2;
        if let Some(name) = RECIPES.get(self.cursor) {
            g.text_bold(dx, top, name, COL_FG, COL_BG);
        }
        g.text(dx, top + 2, "A display-only screen: the sim does", COL_DIM, COL_BG);
        g.text(dx, top + 3, "not simulate crafting (plan §3.2).", COL_DIM, COL_BG);
        if layout.rows >= 2 {
            g.text(2, layout.rows - 2, "j/k: move   Esc/q: back", COL_DIM, COL_BG);
        }
    }

    fn render_look(&mut self, view: &UiView, layout: &Layout) {
        self.panel_frame(layout, "Look around");
        let g = &mut self.grid;
        let (x, y, z) = (self.look_cursor[0], self.look_cursor[1], self.look_cursor[2]);
        g.text(2, 2, &format!("Cursor: ({x}, {y}, {z})"), COL_FG, COL_BG);
        for (i, line) in view.look_lines.iter().enumerate() {
            let row = 4 + i;
            if row + 1 >= layout.rows {
                break;
            }
            g.text(2, row, line, COL_FG, COL_BG);
        }
        if layout.rows >= 2 {
            g.text(2, layout.rows - 2, "Movement keys move the cursor   Esc/q: back", COL_DIM, COL_BG);
        }
    }

    fn render_menu(&mut self, view: &UiView, layout: &Layout) {
        let g = &mut self.grid;
        g.clear(COL_BG);
        let (bx, by, bw, bh) = menu_box(layout);
        g.border(bx, by, bw, bh, COL_ACCENT, COL_BG);
        g.text_bold(bx + 2, by + 1, "CATACLYSM: DARK DAYS AHEAD", COL_ACCENT, COL_BG);
        g.text(bx + 2, by + 2, &format!("sim-server — {}", view.scene_name), COL_DIM, COL_BG);
        for (i, entry) in MENU_ENTRIES.iter().enumerate() {
            let y = by + 4 + i;
            let selected = i == self.cursor;
            let bg = if selected { COL_HILITE_BG } else { COL_BG };
            if selected {
                g.fill_bg(bx + 1, y, bw - 2, 1, COL_HILITE_BG);
            }
            let marker = if selected { ">" } else { " " };
            let fg = if selected { COL_ACCENT } else { COL_FG };
            g.text(bx + 2, y, &format!("{marker} {entry}"), fg, bg);
        }
        g.text(bx + 2, by + bh - 2, "Entries are display-only in the sim.", COL_DIM, COL_BG);
    }
}

/// The main menu's centered box (synthetic: centered half-size
/// panel). Returns (x, y, w, h) in cells.
fn menu_box(layout: &Layout) -> (usize, usize, usize, usize) {
    let bw = (layout.cols / 2).max(10);
    let bh = (layout.rows / 2).max(10);
    (
        (layout.cols - bw) / 2,
        (layout.rows - bh) / 2,
        bw.min(layout.cols),
        bh.min(layout.rows),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_font_hash_is_pinned() {
        // §3.3: UI goldens pin the bundled font file's hash. If
        // this fails, the font file changed — update FONT_FNV1A
        // and the recorded SHA-256 in the module docs together,
        // and expect golden churn.
        assert_eq!(fnv1a(FONT_BYTES), FONT_FNV1A, "bundled font bytes changed");
    }

    #[test]
    fn font_metrics_are_monospace_cells() {
        let f = FontSet::bundled();
        assert!(f.cell_w >= 4 && f.cell_w <= 32, "cell_w {}", f.cell_w);
        assert!(f.cell_h >= 8 && f.cell_h <= 40, "cell_h {}", f.cell_h);
        // Monospace: every printable ASCII glyph shares the
        // advance the cell width was derived from.
        let scaled = f.font.as_scaled(f.scale);
        for ch in ['i', 'W', '█', '─'] {
            let adv = scaled.h_advance(scaled.glyph_id(ch));
            assert!((adv - f.cell_w as f32).abs() <= 1.0, "{ch} advance {adv}");
        }
    }
}
