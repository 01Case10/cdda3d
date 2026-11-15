//! Engine-side input mapping (gates 4.7–4.10 at logic level).
//!
//! Engine events become `proto` `Input` messages. Key codes are
//! platform-independent: printable keys carry their Unicode scalar
//! (so `'i'` is 105, matching the Phase 1 sample), and non-printable
//! keys use the `KEY_*` constants below (>= 0x1000, never colliding
//! with a scalar). The modifier bitmask is the schema's: 1=shift,
//! 2=ctrl, 4=alt, 8=meta (`schema/cdda_ipc.fbs`, `Input.modifiers`).

use proto::{InputKind, InputT, UiMode};

use crate::frame::TerrainRect;
use crate::scaling::{Size2, Viewport, window_click_to_ui};

pub const KEY_ESCAPE: i32 = 0x1001;
pub const KEY_ENTER: i32 = 0x1002;
pub const KEY_TAB: i32 = 0x1003;
pub const KEY_BACKSPACE: i32 = 0x1004;
pub const KEY_DELETE: i32 = 0x1005;
pub const KEY_LEFT: i32 = 0x1010;
pub const KEY_RIGHT: i32 = 0x1011;
pub const KEY_UP: i32 = 0x1012;
pub const KEY_DOWN: i32 = 0x1013;
pub const KEY_HOME: i32 = 0x1014;
pub const KEY_END: i32 = 0x1015;
pub const KEY_PAGE_UP: i32 = 0x1016;
pub const KEY_PAGE_DOWN: i32 = 0x1017;
/// F1..F12 are `KEY_F1 + (n-1)`.
pub const KEY_F1: i32 = 0x1101;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineKey {
    Char(char),
    Escape, Enter, Tab, Backspace, Delete,
    Left, Right, Up, Down, Home, End, PageUp, PageDown,
    Function(u8),
    Space,
}
impl EngineKey {
    pub fn key_code(&self) -> i32 {
        match self {
            EngineKey::Char(c) => *c as i32,
            EngineKey::Space => ' ' as i32,
            EngineKey::Escape => KEY_ESCAPE,
            EngineKey::Enter => KEY_ENTER,
            EngineKey::Tab => KEY_TAB,
            EngineKey::Backspace => KEY_BACKSPACE,
            EngineKey::Delete => KEY_DELETE,
            EngineKey::Left => KEY_LEFT,
            EngineKey::Right => KEY_RIGHT,
            EngineKey::Up => KEY_UP,
            EngineKey::Down => KEY_DOWN,
            EngineKey::Home => KEY_HOME,
            EngineKey::End => KEY_END,
            EngineKey::PageUp => KEY_PAGE_UP,
            EngineKey::PageDown => KEY_PAGE_DOWN,
            EngineKey::Function(n) => KEY_F1 + ((*n).clamp(1, 12) - 1) as i32,
        }
    }
    /// Printable keys also produce a Text event in a real event loop;
    /// this reports the character a Key event stands for, if any.
    pub fn as_char(&self) -> Option<char> {
        match self { EngineKey::Char(c) => Some(*c), EngineKey::Space => Some(' '), _ => None }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Modifiers { pub shift: bool, pub ctrl: bool, pub alt: bool, pub meta: bool }
impl Modifiers {
    pub fn bitmask(&self) -> u32 {
        (self.shift as u32) | ((self.ctrl as u32) << 1) | ((self.alt as u32) << 2) | ((self.meta as u32) << 3)
    }
    pub fn from_bitmask(m: u32) -> Self {
        Self { shift: m & 1 != 0, ctrl: m & 2 != 0, alt: m & 4 != 0, meta: m & 8 != 0 }
    }
}

fn base_input(kind: InputKind) -> InputT {
    // Generated FlatBuffers object types are #[non_exhaustive] outside
    // the proto crate: build via Default + field assignment.
    let mut i = InputT::default();
    i.kind = kind;
    i.button = -1;
    i
}

pub fn key_to_input(key: EngineKey, modifiers: Modifiers, pressed: bool) -> InputT {
    let mut i = base_input(InputKind::Key);
    i.key_code = key.key_code();
    i.modifiers = modifiers.bitmask();
    i.pressed = pressed;
    i
}

/// Text input (gate 4.8): the string travels verbatim, including
/// non-ASCII — FlatBuffers strings are UTF-8, no transcoding here.
pub fn text_to_input(text: &str) -> InputT {
    let mut i = base_input(InputKind::Text);
    i.text = Some(text.to_string());
    i
}

pub fn mouse_move_to_input(ui_x: i32, ui_y: i32, modifiers: Modifiers) -> InputT {
    let mut i = base_input(InputKind::MouseMove);
    i.mouse_x = ui_x; i.mouse_y = ui_y; i.modifiers = modifiers.bitmask();
    i
}

/// `button`: 0=left, 1=middle, 2=right (engine convention; the fork
/// injector maps it onward). Coordinates are UI/display pixels.
pub fn mouse_button_to_input(ui_x: i32, ui_y: i32, button: i32, pressed: bool, modifiers: Modifiers) -> InputT {
    let mut i = base_input(InputKind::MouseButton);
    i.mouse_x = ui_x; i.mouse_y = ui_y; i.button = button; i.pressed = pressed; i.modifiers = modifiers.bitmask();
    i
}

pub fn wheel_to_input(ui_x: i32, ui_y: i32, delta_lines: i32, modifiers: Modifiers) -> InputT {
    let mut i = base_input(InputKind::Wheel);
    i.mouse_x = ui_x; i.mouse_y = ui_y; i.wheel_delta = delta_lines; i.modifiers = modifiers.bitmask();
    i
}

pub fn focus_to_input(focused: bool) -> InputT {
    let mut i = base_input(InputKind::Focus);
    i.focused = focused;
    i
}

// --- UiMode gating + click routing (gates 4.9 / 4.10 / 4.11 separation) ----

#[derive(Debug, Clone, PartialEq)]
pub enum ClickRoute {
    /// Forward to the game as a UI-space mouse event.
    UiClick(InputT),
    /// MapInput mode + inside the terrain rect: a world-pick
    /// candidate — the caller runs picking and sends a PickResult.
    WorldPick { ui_x: i32, ui_y: i32 },
    /// Letterbox / outside the UI quad: nothing to forward.
    Ignored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragRoute {
    /// Forward as UI interaction (unused for drags today; menus use clicks).
    Ui,
    /// Orbit/move the 3D camera (MapInput mode only).
    Camera,
    /// Swallow the drag entirely (UiInput mode, gate 4.9).
    Ignored,
}

#[derive(Debug, Clone)]
pub struct InputRouter {
    pub mode: UiMode,
    pub terrain_rect: Option<TerrainRect>,
    /// Extra UI panel rects outside the sidebar split (e.g. a modal);
    /// clicks there are always UI clicks.
    pub panel_rects: Vec<TerrainRect>,
}
impl InputRouter {
    pub fn new(mode: UiMode, terrain_rect: Option<TerrainRect>) -> Self {
        Self { mode, terrain_rect, panel_rects: Vec::new() }
    }
    /// Gate 4.9: in UiInput mode camera drags are ignored.
    pub fn route_drag(&self) -> DragRoute {
        if self.mode == UiMode::UiInput { DragRoute::Ignored } else { DragRoute::Camera }
    }
    pub fn camera_drag_allowed(&self) -> bool { self.route_drag() == DragRoute::Camera }

    /// Is this UI-space point a world-pick candidate? The separation
    /// rule (gate 4.11): only in MapInput mode, only inside the
    /// reported terrain rect, and never inside a UI panel rect.
    pub fn is_world_pick(&self, ui_x: i32, ui_y: i32) -> bool {
        if self.mode != UiMode::MapInput { return false; }
        if self.panel_rects.iter().any(|r| r.contains(ui_x, ui_y)) { return false; }
        match self.terrain_rect {
            Some(r) => r.contains(ui_x, ui_y),
            None => false,
        }
    }

    /// Route a click given in *window* pixels: convert through the
    /// viewport to UI space first (gate 4.10 — UI clicks stay in
    /// UI/display-buffer coordinates), then apply mode separation.
    pub fn route_window_click(&self, mx: f32, my: f32, viewport: &Viewport, ui_size: Size2, button: i32, pressed: bool, modifiers: Modifiers) -> ClickRoute {
        let Some((ux, uy)) = window_click_to_ui(mx, my, viewport, ui_size) else {
            return ClickRoute::Ignored;
        };
        self.route_ui_click(ux, uy, button, pressed, modifiers)
    }

    pub fn route_ui_click(&self, ui_x: i32, ui_y: i32, button: i32, pressed: bool, modifiers: Modifiers) -> ClickRoute {
        // Only press events pick; releases stay UI events so the game
        // sees balanced button state.
        if pressed && self.is_world_pick(ui_x, ui_y) {
            return ClickRoute::WorldPick { ui_x, ui_y };
        }
        ClickRoute::UiClick(mouse_button_to_input(ui_x, ui_y, button, pressed, modifiers))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_matrix_codes() {
        assert_eq!(key_to_input(EngineKey::Char('i'), Modifiers::default(), true).key_code, 105);
        assert_eq!(key_to_input(EngineKey::Char('5'), Modifiers::default(), true).key_code, '5' as i32);
        assert_eq!(key_to_input(EngineKey::Char('?'), Modifiers::default(), true).key_code, '?' as i32);
        assert_eq!(key_to_input(EngineKey::Left, Modifiers::default(), true).key_code, KEY_LEFT);
        assert_eq!(key_to_input(EngineKey::Function(1), Modifiers::default(), true).key_code, KEY_F1);
        assert_eq!(key_to_input(EngineKey::Function(12), Modifiers::default(), true).key_code, KEY_F1 + 11);
        assert_eq!(key_to_input(EngineKey::Enter, Modifiers::default(), true).key_code, KEY_ENTER);
    }

    #[test]
    fn modifier_bitmask_matches_schema() {
        assert_eq!(Modifiers { shift: true, ..Default::default() }.bitmask(), 1);
        assert_eq!(Modifiers { ctrl: true, ..Default::default() }.bitmask(), 2);
        assert_eq!(Modifiers { alt: true, ..Default::default() }.bitmask(), 4);
        assert_eq!(Modifiers { meta: true, ..Default::default() }.bitmask(), 8);
        let all = Modifiers { shift: true, ctrl: true, alt: true, meta: true };
        assert_eq!(all.bitmask(), 15);
        assert_eq!(Modifiers::from_bitmask(15), all);
    }

    #[test]
    fn text_non_ascii_verbatim() {
        let i = text_to_input("café — ☃ 日本語");
        assert_eq!(i.text.as_deref(), Some("café — ☃ 日本語"));
        assert_eq!(i.kind, InputKind::Text);
    }

    #[test]
    fn ui_mode_gates_camera_and_picks() {
        let tr = TerrainRect { x: 0, y: 0, w: 200, h: 200 };
        let ui = InputRouter::new(UiMode::UiInput, Some(tr));
        assert_eq!(ui.route_drag(), DragRoute::Ignored);
        assert!(!ui.is_world_pick(50, 50));
        let map = InputRouter::new(UiMode::MapInput, Some(tr));
        assert_eq!(map.route_drag(), DragRoute::Camera);
        assert!(map.is_world_pick(50, 50));
        assert!(!map.is_world_pick(250, 50), "outside terrain rect is a panel, not a pick");
    }
}
