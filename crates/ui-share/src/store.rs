//! Double-buffered shared-memory UI store (Phase 4, gates 4.3/4.4 and
//! the real form of gate 1.13).
//!
//! Layout built on top of `proto::shm::UiShmHeader` (48 bytes at
//! offset 0, the cross-language agreement from Phase 1):
//!
//! ```text
//! offset 0                        UiShmHeader (48 B)
//! offset 48                       slot 0: SlotHeader (32 B) + pixels
//! offset 48 + SLOT                slot 1: SlotHeader (32 B) + pixels
//! SLOT = 32 + stride * height     (pixel capacity fixed at creation)
//! ```
//!
//! Each slot header carries its own `seq` and a 64-bit FNV-1a
//! checksum over its pixel bytes plus its dimensions. The writer
//! always writes the *inactive* slot, fills its slot header, and
//! only then publishes by rewriting the main header (new `seq`,
//! flipped `buffer_index`, dirty rect). The reader reads the main
//! header, copies the indicated slot, then re-reads the header and
//! the slot header: any disagreement in `seq`/`buffer_index`, or a
//! checksum mismatch, means the frame was torn or superseded and is
//! rejected — the reader never observes a partial frame. This is
//! the classic double-buffer + sequence-validation protocol; the
//! checksum additionally catches corruption that a pure sequence
//! check cannot (e.g. a writer that died mid-pixel-write without
//! bumping the sequence again).

use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use proto::UiShmHeader;

pub const SLOT_HEADER_SIZE: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirtyRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}
impl DirtyRect {
    pub fn full(width: u32, height: u32) -> Self {
        Self { x: 0, y: 0, w: width as i32, h: height as i32 }
    }
    pub fn is_empty(&self) -> bool { self.w <= 0 || self.h <= 0 }
    pub fn clamped(&self, width: u32, height: u32) -> Self {
        if self.is_empty() { return Self { x: 0, y: 0, w: 0, h: 0 }; }
        let x0 = self.x.clamp(0, width as i32);
        let y0 = self.y.clamp(0, height as i32);
        let x1 = (self.x + self.w).clamp(0, width as i32);
        let y1 = (self.y + self.h).clamp(0, height as i32);
        Self { x: x0, y: y0, w: (x1 - x0).max(0), h: (y1 - y0).max(0) }
    }
    pub fn contains(&self, px: i32, py: i32) -> bool {
        !self.is_empty() && px >= self.x && px < self.x + self.w && py >= self.y && py < self.y + self.h
    }
}

/// FNV-1a 64 over dimensions + pixel bytes.
pub fn frame_checksum(width: u32, height: u32, stride: u32, pixels: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |bytes: &[u8]| {
        for &b in bytes {
            hash ^= b as u64;
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    mix(&width.to_le_bytes());
    mix(&height.to_le_bytes());
    mix(&stride.to_le_bytes());
    mix(pixels);
    hash
}

#[derive(Debug)]
pub enum StoreError {
    Io(io::Error),
    InvalidDimensions,
    PixelBufferTooSmall { needed: usize, got: usize },
    TornFrame(String),
    NoFrameYet,
    BadMagic,
    SeqMismatch { header_seq: u64, slot_seq: u64 },
    ChecksumMismatch { expected: u64, actual: u64 },
}
impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Io(e) => write!(f, "store io: {e}"),
            StoreError::InvalidDimensions => write!(f, "invalid dimensions"),
            StoreError::PixelBufferTooSmall { needed, got } => write!(f, "pixel buffer too small: need {needed}, got {got}"),
            StoreError::TornFrame(m) => write!(f, "torn frame: {m}"),
            StoreError::NoFrameYet => write!(f, "no frame published yet"),
            StoreError::BadMagic => write!(f, "bad shm magic"),
            StoreError::SeqMismatch { header_seq, slot_seq } => write!(f, "seq mismatch: header {header_seq} vs slot {slot_seq}"),
            StoreError::ChecksumMismatch { expected, actual } => write!(f, "checksum mismatch: expected {expected:#x}, actual {actual:#x}"),
        }
    }
}
impl std::error::Error for StoreError {}
impl From<io::Error> for StoreError { fn from(e: io::Error) -> Self { StoreError::Io(e) } }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameSnapshot {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub seq: u64,
    pub buffer_index: u8,
    pub dirty: DirtyRect,
    pub checksum: u64,
    /// Row-major RGBA8, `stride * height` bytes (rows may be padded).
    pub pixels: Vec<u8>,
}
impl FrameSnapshot {
    /// Tightly-packed RGBA copy (padding stripped).
    pub fn tight_pixels(&self) -> Vec<u8> {
        let row = (self.width * 4) as usize;
        let mut out = Vec::with_capacity(row * self.height as usize);
        for y in 0..self.height as usize {
            let s = y * self.stride as usize;
            out.extend_from_slice(&self.pixels[s..s + row]);
        }
        out
    }
    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let o = (y * self.stride + x * 4) as usize;
        [self.pixels[o], self.pixels[o + 1], self.pixels[o + 2], self.pixels[o + 3]]
    }
}

// --- backing ---------------------------------------------------------------

#[derive(Clone)]
enum Backing {
    Memory(Arc<RwLock<Vec<u8>>>),
    File(Arc<Mutex<File>>, Arc<Mutex<PathBuf>>, usize),
}
impl Backing {
    fn len(&self) -> usize {
        match self { Backing::Memory(v) => v.read().unwrap().len(), Backing::File(_, _, n) => *n }
    }
    fn read(&self, offset: usize, buf: &mut [u8]) -> Result<(), StoreError> {
        match self {
            Backing::Memory(v) => {
                let v = v.read().unwrap();
                if offset + buf.len() > v.len() { return Err(StoreError::TornFrame("read out of bounds".into())); }
                buf.copy_from_slice(&v[offset..offset + buf.len()]);
                Ok(())
            }
            Backing::File(f, _, _) => {
                use std::os::unix::fs::FileExt;
                let f = f.lock().unwrap();
                f.read_exact_at(buf, offset as u64)?;
                Ok(())
            }
        }
    }
    fn write(&self, offset: usize, data: &[u8]) -> Result<(), StoreError> {
        match self {
            Backing::Memory(v) => {
                let mut v = v.write().unwrap();
                if offset + data.len() > v.len() { return Err(StoreError::TornFrame("write out of bounds".into())); }
                v[offset..offset + data.len()].copy_from_slice(data);
                Ok(())
            }
            Backing::File(f, _, _) => {
                use std::os::unix::fs::FileExt;
                let f = f.lock().unwrap();
                f.write_all_at(data, offset as u64)?;
                Ok(())
            }
        }
    }
    fn sync(&self) -> Result<(), StoreError> {
        if let Backing::File(f, _, _) = self { f.lock().unwrap().sync_data()?; }
        Ok(())
    }
}

fn slot_header_bytes(seq: u64, checksum: u64, w: u32, h: u32, stride: u32) -> [u8; SLOT_HEADER_SIZE] {
    let mut b = [0u8; SLOT_HEADER_SIZE];
    b[0..8].copy_from_slice(&seq.to_le_bytes());
    b[8..16].copy_from_slice(&checksum.to_le_bytes());
    b[16..20].copy_from_slice(&w.to_le_bytes());
    b[20..24].copy_from_slice(&h.to_le_bytes());
    b[24..28].copy_from_slice(&stride.to_le_bytes());
    b
}
fn parse_slot_header(b: &[u8]) -> (u64, u64, u32, u32, u32) {
    (
        u64::from_le_bytes(b[0..8].try_into().unwrap()),
        u64::from_le_bytes(b[8..16].try_into().unwrap()),
        u32::from_le_bytes(b[16..20].try_into().unwrap()),
        u32::from_le_bytes(b[20..24].try_into().unwrap()),
        u32::from_le_bytes(b[24..28].try_into().unwrap()),
    )
}

/// A handle onto the shared store. Clone it to get a second handle
/// onto the same backing (writer/reader pair in-process; the file
/// form is the cross-process form).
#[derive(Clone)]
pub struct UiShmStore {
    backing: Backing,
    width: u32,
    height: u32,
    stride: u32,
    slot_size: usize,
    next_seq: Arc<Mutex<u64>>,
}

impl UiShmStore {
    fn total_size(width: u32, height: u32, stride: u32) -> usize {
        UiShmHeader::SIZE + 2 * (SLOT_HEADER_SIZE + (stride * height) as usize)
    }
    fn new(backing: Backing, width: u32, height: u32, stride: u32) -> Self {
        let slot_size = SLOT_HEADER_SIZE + (stride * height) as usize;
        Self { backing, width, height, stride, slot_size, next_seq: Arc::new(Mutex::new(0)) }
    }

    pub fn create_memory(width: u32, height: u32) -> Result<Self, StoreError> {
        if width == 0 || height == 0 { return Err(StoreError::InvalidDimensions); }
        let stride = width * 4;
        let total = Self::total_size(width, height, stride);
        let mut data = vec![0u8; total];
        let hdr = UiShmHeader::new(width, height, stride);
        data[0..UiShmHeader::SIZE].copy_from_slice(&hdr.to_bytes());
        Ok(Self::new(Backing::Memory(Arc::new(RwLock::new(data))), width, height, stride))
    }

    /// File-backed store using only `std` (`read_at`/`write_at`).
    pub fn create_file(path: &Path, width: u32, height: u32) -> Result<Self, StoreError> {
        if width == 0 || height == 0 { return Err(StoreError::InvalidDimensions); }
        let stride = width * 4;
        let total = Self::total_size(width, height, stride);
        let file = OpenOptions::new().read(true).write(true).create(true).truncate(true).open(path)?;
        file.set_len(total as u64)?;
        let backing = Backing::File(Arc::new(Mutex::new(file)), Arc::new(Mutex::new(path.to_path_buf())), total);
        let store = Self::new(backing, width, height, stride);
        let hdr = UiShmHeader::new(width, height, stride);
        store.backing.write(0, &hdr.to_bytes())?;
        store.backing.sync()?;
        Ok(store)
    }

    /// Open an existing file-backed store, reading geometry from its header.
    pub fn open_file(path: &Path) -> Result<Self, StoreError> {
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        let len = file.metadata()?.len() as usize;
        let backing = Backing::File(Arc::new(Mutex::new(file)), Arc::new(Mutex::new(path.to_path_buf())), len);
        let mut hb = [0u8; UiShmHeader::SIZE];
        backing.read(0, &mut hb)?;
        let hdr = UiShmHeader::from_bytes(&hb).ok_or(StoreError::BadMagic)?;
        let store = Self::new(backing, hdr.width, hdr.height, hdr.stride);
        *store.next_seq.lock().unwrap() = hdr.seq;
        Ok(store)
    }

    pub fn dimensions(&self) -> (u32, u32, u32) { (self.width, self.height, self.stride) }
    pub fn backing_len(&self) -> usize { self.backing.len() }
    fn slot_offset(&self, buffer_index: u8) -> usize {
        UiShmHeader::SIZE + buffer_index as usize * self.slot_size
    }

    pub fn read_header(&self) -> Result<UiShmHeader, StoreError> {
        let mut hb = [0u8; UiShmHeader::SIZE];
        self.backing.read(0, &mut hb)?;
        UiShmHeader::from_bytes(&hb).ok_or(StoreError::BadMagic)
    }

    /// Writer side: publish a new frame into the inactive slot.
    /// `pixels` is RGBA8 with row pitch `stride` (use `width*4` for
    /// tightly packed input and pass that as the stride via
    /// `write_frame_tight` instead if your stride differs).
    pub fn write_frame(&self, pixels: &[u8], dirty: DirtyRect) -> Result<u64, StoreError> {
        self.write_frame_with_stride(pixels, self.width, self.height, self.stride, dirty)
    }

    pub fn write_frame_tight(&self, pixels: &[u8], width: u32, height: u32, dirty: DirtyRect) -> Result<u64, StoreError> {
        // Expand tight rows into the store stride if needed.
        if width == self.width && height == self.height && width * 4 == self.stride {
            return self.write_frame_with_stride(pixels, width, height, self.stride, dirty);
        }
        if width != self.width || height != self.height { return Err(StoreError::InvalidDimensions); }
        let mut padded = vec![0u8; (self.stride * self.height) as usize];
        for y in 0..height as usize {
            let s = y * (width * 4) as usize;
            let d = y * self.stride as usize;
            padded[d..d + (width * 4) as usize].copy_from_slice(&pixels[s..s + (width * 4) as usize]);
        }
        self.write_frame_with_stride(&padded, width, height, self.stride, dirty)
    }

    pub fn write_frame_with_stride(&self, pixels: &[u8], width: u32, height: u32, stride: u32, dirty: DirtyRect) -> Result<u64, StoreError> {
        if width != self.width || height != self.height || stride != self.stride {
            return Err(StoreError::InvalidDimensions);
        }
        let needed = (stride * height) as usize;
        if pixels.len() < needed { return Err(StoreError::PixelBufferTooSmall { needed, got: pixels.len() }); }
        let pixels = &pixels[..needed];
        let cur = self.read_header().unwrap_or(UiShmHeader::new(width, height, stride));
        let target: u8 = if cur.seq == 0 { 0 } else { 1 - (cur.buffer_index & 1) };
        let seq = {
            let mut n = self.next_seq.lock().unwrap();
            *n = (*n).max(cur.seq) + 1;
            *n
        };
        let checksum = frame_checksum(width, height, stride, pixels);
        let off = self.slot_offset(target);
        // 1) pixels, 2) slot header, 3) main header publishes.
        self.backing.write(off + SLOT_HEADER_SIZE, pixels)?;
        self.backing.write(off, &slot_header_bytes(seq, checksum, width, height, stride))?;
        let mut hdr = UiShmHeader::new(width, height, stride);
        hdr.seq = seq;
        hdr.buffer_index = target;
        let d = dirty.clamped(width, height);
        hdr.dirty_x = d.x; hdr.dirty_y = d.y; hdr.dirty_w = d.w; hdr.dirty_h = d.h;
        self.backing.write(0, &hdr.to_bytes())?;
        self.backing.sync()?;
        Ok(seq)
    }

    /// Reader side: copy out the newest complete frame, validating
    /// sequence (before/after) and checksum. Retries a few times if
    /// the writer publishes concurrently; a persistently inconsistent
    /// frame is an error, never a torn success.
    pub fn read_latest(&self) -> Result<FrameSnapshot, StoreError> {
        let mut last_err: Option<StoreError> = None;
        for _ in 0..8 {
            match self.try_read_once() {
                Ok(f) => return Ok(f),
                Err(e) => last_err = Some(e),
            }
        }
        Err(last_err.unwrap_or(StoreError::NoFrameYet))
    }

    fn try_read_once(&self) -> Result<FrameSnapshot, StoreError> {
        let hdr1 = self.read_header()?;
        if hdr1.seq == 0 { return Err(StoreError::NoFrameYet); }
        let idx = hdr1.buffer_index & 1;
        let off = self.slot_offset(idx);
        let mut sb = [0u8; SLOT_HEADER_SIZE];
        self.backing.read(off, &mut sb)?;
        let (slot_seq, checksum, w, h, stride) = parse_slot_header(&sb);
        if slot_seq != hdr1.seq {
            return Err(StoreError::SeqMismatch { header_seq: hdr1.seq, slot_seq });
        }
        if (w, h, stride) != (hdr1.width, hdr1.height, hdr1.stride) {
            return Err(StoreError::TornFrame("slot geometry disagrees with header".into()));
        }
        let plen = (stride * h) as usize;
        let mut pixels = vec![0u8; plen];
        self.backing.read(off + SLOT_HEADER_SIZE, &mut pixels)?;
        let hdr2 = self.read_header()?;
        if hdr2.seq != hdr1.seq || hdr2.buffer_index != hdr1.buffer_index {
            return Err(StoreError::TornFrame("header changed during read".into()));
        }
        let actual = frame_checksum(w, h, stride, &pixels);
        if actual != checksum {
            return Err(StoreError::ChecksumMismatch { expected: checksum, actual });
        }
        Ok(FrameSnapshot {
            width: w, height: h, stride, seq: hdr1.seq, buffer_index: idx,
            dirty: DirtyRect { x: hdr1.dirty_x, y: hdr1.dirty_y, w: hdr1.dirty_w, h: hdr1.dirty_h },
            checksum, pixels,
        })
    }

    /// Test/debug hook: overwrite one backing byte (simulates a torn write).
    pub fn debug_corrupt_byte(&self, offset: usize, value: u8) -> Result<(), StoreError> {
        self.backing.write(offset, &[value])
    }
    /// Offset of a slot's pixel data — for torn-frame tests.
    pub fn debug_slot_pixel_offset(&self, buffer_index: u8) -> usize {
        self.slot_offset(buffer_index) + SLOT_HEADER_SIZE
    }
}
