//! `UiFrame` notification handling: validation against the store,
//! map-region (terrain rect) masking (gate 4.2), and dirty-rect
//! upload planning (gate 4.5).
//!
//! Terminology: a *UiFrame notification* is the small FlatBuffers
//! `UiFrame` message on the socket saying "a new UI frame is ready
//! in shared memory, in buffer N, sequence S". The pixels themselves
//! never travel on the socket — they are read from the shared store.

use proto::{UiFrameT, UiMode};

use crate::store::{DirtyRect, FrameSnapshot, UiShmStore};

/// The terrain window's rectangle in UI (display-buffer) pixels, as
/// reported by the game in every `UiFrame`. Inside it the engine
/// draws the 3D world, so the composited UI must be transparent
/// there; outside it the game's panels/menus show verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerrainRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}
impl TerrainRect {
    pub fn from_frame(f: &UiFrameT) -> Option<Self> {
        if f.terrain_rect_w <= 0 || f.terrain_rect_h <= 0 || f.terrain_rect_x < 0 || f.terrain_rect_y < 0 {
            return None;
        }
        Some(Self { x: f.terrain_rect_x, y: f.terrain_rect_y, w: f.terrain_rect_w, h: f.terrain_rect_h })
    }
    pub fn as_dirty(&self) -> DirtyRect { DirtyRect { x: self.x, y: self.y, w: self.w, h: self.h } }
    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.x + self.w && py >= self.y && py < self.y + self.h
    }
    pub fn clamped(&self, width: u32, height: u32) -> Option<Self> {
        let d = self.as_dirty().clamped(width, height);
        if d.is_empty() { None } else { Some(Self { x: d.x, y: d.y, w: d.w, h: d.h }) }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameError {
    BufferIndexOutOfRange(u8),
    SeqMismatch { frame_seq: u64, store_seq: u64 },
    DimensionMismatch { frame: (u32, u32, u32), store: (u32, u32, u32) },
    BufferMismatch { frame_index: u8, store_index: u8 },
}
impl std::fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameError::BufferIndexOutOfRange(i) => write!(f, "buffer_index {i} out of range"),
            FrameError::SeqMismatch { frame_seq, store_seq } => write!(f, "UiFrame seq {frame_seq} != store seq {store_seq}"),
            FrameError::DimensionMismatch { frame, store } => write!(f, "UiFrame dims {frame:?} != store dims {store:?}"),
            FrameError::BufferMismatch { frame_index, store_index } => write!(f, "UiFrame buffer {frame_index} != store buffer {store_index}"),
        }
    }
}
impl std::error::Error for FrameError {}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedFrame {
    pub seq: u64,
    pub buffer_index: u8,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub dirty: DirtyRect,
    pub mode: UiMode,
    pub terrain_rect: Option<TerrainRect>,
}

/// Validate a `UiFrame` notification against the store's current
/// header. A stale notification (seq behind the store) is a mismatch,
/// not a frame to upload.
pub fn validate_frame(store: &UiShmStore, frame: &UiFrameT) -> Result<ValidatedFrame, FrameError> {
    if frame.buffer_index > 1 {
        return Err(FrameError::BufferIndexOutOfRange(frame.buffer_index));
    }
    let hdr = store.read_header().map_err(|_| FrameError::SeqMismatch { frame_seq: frame.seq, store_seq: 0 })?;
    if frame.seq != hdr.seq {
        return Err(FrameError::SeqMismatch { frame_seq: frame.seq, store_seq: hdr.seq });
    }
    if frame.buffer_index != hdr.buffer_index {
        return Err(FrameError::BufferMismatch { frame_index: frame.buffer_index, store_index: hdr.buffer_index });
    }
    let store_dims = (hdr.width, hdr.height, hdr.stride);
    let frame_dims = (frame.width, frame.height, frame.stride);
    if frame_dims != store_dims {
        return Err(FrameError::DimensionMismatch { frame: frame_dims, store: store_dims });
    }
    Ok(ValidatedFrame {
        seq: frame.seq,
        buffer_index: frame.buffer_index,
        width: frame.width,
        height: frame.height,
        stride: frame.stride,
        dirty: DirtyRect { x: frame.dirty_x, y: frame.dirty_y, w: frame.dirty_w, h: frame.dirty_h },
        mode: frame.mode,
        terrain_rect: TerrainRect::from_frame(frame),
    })
}

/// Build the `UiFrameT` a writer would send for a snapshot (used by
/// the harness and by the fork-side contract tests).
pub fn frame_notification(snap: &FrameSnapshot, mode: UiMode, terrain: Option<TerrainRect>) -> UiFrameT {
    let (tx, ty, tw, th) = match terrain {
        Some(r) => (r.x, r.y, r.w, r.h),
        None => (-1, -1, -1, -1),
    };
    let mut f = UiFrameT::default();
    f.buffer_index = snap.buffer_index;
    f.width = snap.width;
    f.height = snap.height;
    f.stride = snap.stride;
    f.seq = snap.seq;
    f.dirty_x = snap.dirty.x;
    f.dirty_y = snap.dirty.y;
    f.dirty_w = snap.dirty.w;
    f.dirty_h = snap.dirty.h;
    f.mode = mode;
    f.terrain_rect_x = tx;
    f.terrain_rect_y = ty;
    f.terrain_rect_w = tw;
    f.terrain_rect_h = th;
    f}

// --- masking (gate 4.2) ----------------------------------------------------

/// Set alpha to 0 for every pixel inside `terrain` (clamped to the
/// frame). Pixels outside are left byte-identical. RGB inside the
/// rect is preserved (only alpha matters to the blend), matching how
/// a cleared terrain window would arrive from Route B capture.
pub fn apply_terrain_mask(pixels: &mut [u8], width: u32, height: u32, stride: u32, terrain: Option<TerrainRect>) {
    let Some(rect) = terrain.and_then(|r| r.clamped(width, height)) else { return };
    for y in rect.y..rect.y + rect.h {
        for x in rect.x..rect.x + rect.w {
            let o = (y as u32 * stride + x as u32 * 4 + 3) as usize;
            if o < pixels.len() { pixels[o] = 0; }
        }
    }
}

/// Masked copy of a snapshot's tight pixels, ready for upload.
pub fn masked_tight_pixels(snap: &FrameSnapshot, terrain: Option<TerrainRect>) -> Vec<u8> {
    let mut px = snap.tight_pixels();
    // tight stride == width*4
    apply_terrain_mask(&mut px, snap.width, snap.height, snap.width * 4, terrain);
    px
}

// --- dirty-rect upload planning (gate 4.5) ---------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UploadPlan {
    pub rect: DirtyRect,
    pub is_full: bool,
    pub bytes: usize,
}
impl UploadPlan {
    pub fn for_frame(width: u32, height: u32, dirty: DirtyRect) -> Self {
        let d = dirty.clamped(width, height);
        let is_full = d.x == 0 && d.y == 0 && d.w == width as i32 && d.h == height as i32;
        Self { rect: d, is_full, bytes: (d.w.max(0) * d.h.max(0) * 4) as usize }
    }
    pub fn full(width: u32, height: u32) -> Self {
        Self::for_frame(width, height, DirtyRect::full(width, height))
    }
}

/// Apply an upload plan CPU-side: copy only the planned rect from
/// `src` (tight RGBA) into `dst` (tight RGBA, the resident texture
/// copy). Gate 4.5's invariant: if `dst` held the previous frame and
/// `src` is the new frame whose only changes lie inside the dirty
/// rect, the result is pixel-identical to a full upload.
pub fn apply_upload_plan(dst: &mut [u8], src: &[u8], width: u32, height: u32, plan: &UploadPlan) {
    let _ = height;
    let row = (width * 4) as usize;
    let r = plan.rect;
    if r.is_empty() { return; }
    for y in r.y..r.y + r.h {
        let s = (y as usize) * row + (r.x as usize) * 4;
        let n = (r.w as usize) * 4;
        dst[s..s + n].copy_from_slice(&src[s..s + n]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_rect_unknown_when_negative() {
        let mut f = UiFrameT::default();
        f.terrain_rect_x = -1; f.terrain_rect_w = -1;
        assert_eq!(TerrainRect::from_frame(&f), None);
    }

    #[test]
    fn mask_only_touches_terrain_alpha() {
        let (w, h) = (4u32, 4u32);
        let mut px = vec![200u8; (w * h * 4) as usize];
        apply_terrain_mask(&mut px, w, h, w * 4, Some(TerrainRect { x: 1, y: 1, w: 2, h: 2 }));
        for y in 0..4 { for x in 0..4 {
            let a = px[((y * w + x) * 4 + 3) as usize];
            if (1..3).contains(&x) && (1..3).contains(&y) { assert_eq!(a, 0, "({x},{y})"); }
            else { assert_eq!(a, 200, "({x},{y})"); }
        }}
    }
}
