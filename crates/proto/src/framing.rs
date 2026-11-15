//! Length-prefixed framing: u32 LE length + payload.
//!
//! The length prefix is validated against MAX_FRAME_LEN *before* any buffer
//! is sized by it, so an absurd prefix is rejected without a large
//! allocation (gate 1.7).

use crate::{ProtoError, Result};

/// Hard cap on one frame's payload. Snapshots of the view window are budgeted
/// under 2 MB (plan §11) and TileDefs for large mod sets stay well under
/// this; 32 MiB is generous headroom, not an allocation target.
pub const MAX_FRAME_LEN: u32 = 32 * 1024 * 1024;

pub struct FrameEncoder;

impl FrameEncoder {
    /// Frame one payload for the wire.
    pub fn encode(payload: &[u8]) -> Vec<u8> {
        assert!(
            payload.len() <= MAX_FRAME_LEN as usize,
            "payload too large to frame"
        );
        let mut out = Vec::with_capacity(4 + payload.len());
        out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        out.extend_from_slice(payload);
        out
    }
}

/// Incremental decoder: feed arbitrary chunks (frames may be split across
/// reads or coalesced into one read — gate 1.6), pull complete payloads out.
#[derive(Default)]
pub struct FrameDecoder {
    buf: Vec<u8>,
}

impl FrameDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, data: &[u8]) {
        self.buf.extend_from_slice(data);
    }

    /// Number of buffered, not-yet-consumed bytes.
    pub fn buffered_len(&self) -> usize {
        self.buf.len()
    }

    /// Take the next complete frame payload, if one is buffered.
    ///
    /// Errors only on an oversize length prefix; the decoder should be
    /// dropped (connection closed) after that error.
    pub fn next_frame(&mut self) -> Result<Option<Vec<u8>>> {
        if self.buf.len() < 4 {
            return Ok(None);
        }
        let len = u32::from_le_bytes([self.buf[0], self.buf[1], self.buf[2], self.buf[3]]);
        if len > MAX_FRAME_LEN {
            // Reject from the prefix alone — never allocate `len` bytes.
            return Err(ProtoError::OversizeFrame(len));
        }
        let len = len as usize;
        if self.buf.len() < 4 + len {
            return Ok(None);
        }
        let payload = self.buf[4..4 + len].to_vec();
        self.buf.drain(..4 + len);
        Ok(Some(payload))
    }
}
