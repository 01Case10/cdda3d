//! Latency accounting (gate 4.12, harness form).
//!
//! The gate's real form — key press to updated UI frame *on screen*
//! in the running game — needs the fork in-game and a window; both
//! are blocked here. What is measured instead is the engine-side
//! segment this crate owns: frame-ready (store publish) -> store
//! read -> GPU upload -> composite -> readback, on this VM's
//! llvmpipe software adapter. These numbers are a floor for the
//! segment, labeled llvmpipe, and must never be quoted as
//! target-hardware or end-to-end figures.

use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LatencyStats {
    pub store_read: Duration,
    pub upload: Duration,
    pub composite_and_readback: Duration,
}
impl LatencyStats {
    pub fn total(&self) -> Duration { self.store_read + self.upload + self.composite_and_readback }
    pub fn total_ms(&self) -> f64 { self.total().as_secs_f64() * 1000.0 }
}

/// Small timer helper for the harness stages.
pub struct StageTimer { start: Instant }
impl StageTimer {
    pub fn start() -> Self { Self { start: Instant::now() } }
    pub fn elapsed(&self) -> Duration { self.start.elapsed() }
    pub fn restart(&mut self) -> Duration { let e = self.start.elapsed(); self.start = Instant::now(); e }
}

pub fn median(mut samples: Vec<f64>) -> f64 {
    if samples.is_empty() { return 0.0; }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = samples.len();
    if n % 2 == 1 { samples[n / 2] } else { (samples[n / 2 - 1] + samples[n / 2]) / 2.0 }
}
pub fn percentile(mut samples: Vec<f64>, p: f64) -> f64 {
    if samples.is_empty() { return 0.0; }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = ((p / 100.0) * (samples.len() - 1) as f64).round() as usize;
    samples[idx.min(samples.len() - 1)]
}

#[derive(Debug, Clone, PartialEq)]
pub struct LatencyReport {
    pub samples_ms: Vec<f64>,
    pub median_ms: f64,
    pub p99_ms: f64,
    /// Always names the adapter class the numbers came from.
    pub label: String,
}
impl LatencyReport {
    pub fn from_samples(samples_ms: Vec<f64>, adapter_name: &str) -> Self {
        Self {
            median_ms: median(samples_ms.clone()),
            p99_ms: percentile(samples_ms.clone(), 99.0),
            samples_ms,
            label: format!("llvmpipe/software adapter ({adapter_name}), VM 1.5 CPU — NOT target hardware, NOT end-to-end"),
        }
    }
    /// Gate 4.12 budgets (median < 50 ms, p99 < 100 ms) applied to
    /// the harness segment only — passing here is necessary-shaped
    /// evidence, not the gate itself.
    pub fn within_gate_budgets(&self) -> bool { self.median_ms < 50.0 && self.p99_ms < 100.0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn median_and_p99() {
        assert_eq!(median(vec![3.0, 1.0, 2.0]), 2.0);
        assert_eq!(median(vec![1.0, 2.0, 3.0, 4.0]), 2.5);
        assert!(percentile((0..100).map(|i| i as f64).collect(), 99.0) >= 98.0);
    }
}
