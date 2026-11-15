//! Shared-memory UI texture header (plan §6). The layout is defined here, in
//! the proto crate, so the Rust engine and the C++ fork/stub agree on it
//! byte-for-byte. The C++ side static_asserts the same size/offsets
//! (tools/stub-server/stub_server.cpp, gate 1.13).
//!
//! Layout (repr(C), 48 bytes, align 8):
//!   0  magic: u32          = UI_SHM_MAGIC
//!   4  width: u32          pixels
//!   8  height: u32         pixels
//!  12  stride: u32         bytes per row
//!  16  seq: u64            frame sequence counter (writer bumps per frame)
//!  24  dirty_x: i32
//!  28  dirty_y: i32
//!  32  dirty_w: i32
//!  36  dirty_h: i32
//!  40  buffer_index: u8    which of the two buffers is newest (0 or 1)
//!  41  _pad: [u8; 7]

/// "CDAU" as a little-endian u32.
pub const UI_SHM_MAGIC: u32 = 0x4344_4155;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiShmHeader {
    pub magic: u32,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub seq: u64,
    pub dirty_x: i32,
    pub dirty_y: i32,
    pub dirty_w: i32,
    pub dirty_h: i32,
    pub buffer_index: u8,
    pub _pad: [u8; 7],
}

impl UiShmHeader {
    pub const SIZE: usize = 48;

    pub fn new(width: u32, height: u32, stride: u32) -> Self {
        Self {
            magic: UI_SHM_MAGIC,
            width,
            height,
            stride,
            seq: 0,
            dirty_x: 0,
            dirty_y: 0,
            dirty_w: width as i32,
            dirty_h: height as i32,
            buffer_index: 0,
            _pad: [0; 7],
        }
    }

    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        // repr(C) with no padding beyond the explicit _pad, so a plain copy
        // of the struct image is the wire/shared layout.
        let mut out = [0u8; Self::SIZE];
        out[0..4].copy_from_slice(&self.magic.to_le_bytes());
        out[4..8].copy_from_slice(&self.width.to_le_bytes());
        out[8..12].copy_from_slice(&self.height.to_le_bytes());
        out[12..16].copy_from_slice(&self.stride.to_le_bytes());
        out[16..24].copy_from_slice(&self.seq.to_le_bytes());
        out[24..28].copy_from_slice(&self.dirty_x.to_le_bytes());
        out[28..32].copy_from_slice(&self.dirty_y.to_le_bytes());
        out[32..36].copy_from_slice(&self.dirty_w.to_le_bytes());
        out[36..40].copy_from_slice(&self.dirty_h.to_le_bytes());
        out[40] = self.buffer_index;
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::SIZE {
            return None;
        }
        let rd32 = |o: usize| u32::from_le_bytes(bytes[o..o + 4].try_into().unwrap());
        let rdi32 = |o: usize| i32::from_le_bytes(bytes[o..o + 4].try_into().unwrap());
        let hdr = Self {
            magic: rd32(0),
            width: rd32(4),
            height: rd32(8),
            stride: rd32(12),
            seq: u64::from_le_bytes(bytes[16..24].try_into().unwrap()),
            dirty_x: rdi32(24),
            dirty_y: rdi32(28),
            dirty_w: rdi32(32),
            dirty_h: rdi32(36),
            buffer_index: bytes[40],
            _pad: [0; 7],
        };
        if hdr.magic != UI_SHM_MAGIC {
            return None;
        }
        Some(hdr)
    }
}
