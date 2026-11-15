//! Seq/epoch tracking (plan §6 determinism) and heartbeat timeout logic.
//! Both are pure and clock-injected so they are unit-testable without I/O.

/// What to do with an incoming Delta (gate 1.14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeltaDecision {
    /// In-order delta for the current epoch: apply it.
    Apply,
    /// Duplicate or stale delta: discard without applying (idempotent).
    Discard,
    /// A seq was skipped (or no snapshot yet): request a fresh Snapshot and
    /// do not apply this delta.
    NeedsSnapshot,
}

/// Tracks the last applied (epoch, seq). `epoch` increments on every fresh
/// Snapshot; a Delta names the epoch and the seq it applies to.
#[derive(Debug, Default)]
pub struct SeqTracker {
    epoch: Option<u64>,
    /// Last seq applied within the current epoch.
    last_seq: u64,
}

impl SeqTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn epoch(&self) -> Option<u64> {
        self.epoch
    }

    pub fn last_seq(&self) -> u64 {
        self.last_seq
    }

    /// Record a fresh Snapshot: it becomes the new base state. A snapshot
    /// from an older epoch than the current one is stale and ignored.
    /// Returns true if accepted.
    pub fn accept_snapshot(&mut self, seq: u64, epoch: u64) -> bool {
        if let Some(cur) = self.epoch {
            if epoch < cur {
                return false;
            }
        }
        self.epoch = Some(epoch);
        self.last_seq = seq;
        true
    }

    /// Decide what to do with a Delta carrying (seq, epoch).
    pub fn observe_delta(&mut self, seq: u64, epoch: u64) -> DeltaDecision {
        let cur = match self.epoch {
            None => return DeltaDecision::NeedsSnapshot,
            Some(e) => e,
        };
        if epoch < cur {
            return DeltaDecision::Discard; // old epoch
        }
        if epoch > cur {
            return DeltaDecision::NeedsSnapshot; // snapshot we never saw
        }
        if seq <= self.last_seq {
            return DeltaDecision::Discard; // duplicate or out-of-order old
        }
        if seq == self.last_seq + 1 {
            self.last_seq = seq;
            return DeltaDecision::Apply;
        }
        // seq skipped ahead: gap. Do not advance — a fresh snapshot resyncs.
        DeltaDecision::NeedsSnapshot
    }
}

/// Pure heartbeat logic: the caller supplies millisecond timestamps.
#[derive(Debug)]
pub struct HeartbeatTracker {
    pub timeout_ms: u64,
    pub interval_ms: u64,
    last_seen_ms: u64,
    last_ping_ms: u64,
}

impl HeartbeatTracker {
    /// Plan §6: heartbeat every second. Timeout defaults to 3 missed beats.
    pub fn new(now_ms: u64) -> Self {
        Self {
            timeout_ms: 3_000,
            interval_ms: 1_000,
            last_seen_ms: now_ms,
            last_ping_ms: now_ms,
        }
    }

    pub fn with_timeout(now_ms: u64, timeout_ms: u64, interval_ms: u64) -> Self {
        Self {
            timeout_ms,
            interval_ms,
            last_seen_ms: now_ms,
            last_ping_ms: now_ms,
        }
    }

    /// Any message from the peer (Pong or otherwise) counts as alive.
    pub fn note_seen(&mut self, now_ms: u64) {
        self.last_seen_ms = now_ms;
    }

    pub fn note_ping_sent(&mut self, now_ms: u64) {
        self.last_ping_ms = now_ms;
    }

    pub fn should_ping(&self, now_ms: u64) -> bool {
        now_ms.saturating_sub(self.last_ping_ms) >= self.interval_ms
    }

    /// Gate 1.10: peer is dead once no message has been seen for timeout_ms.
    pub fn is_timed_out(&self, now_ms: u64) -> bool {
        now_ms.saturating_sub(self.last_seen_ms) > self.timeout_ms
    }
}
